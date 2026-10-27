package tokyo.runo.maidcafese

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings
import android.util.Log
import androidx.core.content.FileProvider
import java.io.File
import java.net.HttpURLConnection
import java.net.URL
import java.security.MessageDigest

/**
 * 起動のたびに、GitHubの最新リリースを確認して、新しい版があればAPKを取得してインストール画面を開く。
 * Androidの仕様上、インストールの最終確認(「更新」ボタン)は端末の画面でユーザーが押す(無人で入れ替えることはできない)。
 * 配布APKはリリースの`SHA256SUMS.txt`と照合してからインストールする。
 */
object Updater {
    private const val TAG = "MaidCafeSeUpdate"
    private const val API = "https://api.github.com/repos/aon-co-jp/maid-cafe-se/releases/latest"

    data class Release(val version: String, val apkUrl: String, val apkName: String, val sumsUrl: String?)

    sealed interface Result {
        data object UpToDate : Result
        data class Failed(val reason: String) : Result
        /** インストール画面を開いた(または「不明なアプリのインストール」の許可画面を開いた)。 */
        data class Launched(val version: String, val needsPermission: Boolean) : Result
    }

    /** `0.5.3` / `v0.5.3` を数の並びに直して比べる。 */
    fun isNewer(latest: String, current: String): Boolean {
        fun parts(s: String) = s.removePrefix("v").split(".", "-").map { it.takeWhile(Char::isDigit).toIntOrNull() ?: 0 }
        val a = parts(latest)
        val b = parts(current)
        for (i in 0 until maxOf(a.size, b.size)) {
            val x = a.getOrElse(i) { 0 }
            val y = b.getOrElse(i) { 0 }
            if (x != y) return x > y
        }
        return false
    }

    fun currentVersion(ctx: Context): String =
        ctx.packageManager.getPackageInfo(ctx.packageName, 0).versionName.orEmpty()

    /** 通信するので、バックグラウンドのスレッドから呼ぶこと。 */
    fun checkAndUpdate(ctx: Context): Result {
        return try {
            val rel = latestRelease() ?: return Result.UpToDate
            if (!isNewer(rel.version, currentVersion(ctx))) return Result.UpToDate
            val dir = File(ctx.cacheDir, "update").apply { mkdirs() }
            dir.listFiles()?.forEach { it.delete() }
            val apk = File(dir, rel.apkName)
            download(rel.apkUrl, apk)
            verify(rel, apk)
            val needsPermission = Build.VERSION.SDK_INT >= 26 && !ctx.packageManager.canRequestPackageInstalls()
            if (needsPermission) {
                ctx.startActivity(
                    Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES, Uri.parse("package:${ctx.packageName}"))
                        .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
                )
            } else {
                val uri = FileProvider.getUriForFile(ctx, "${ctx.packageName}.updates", apk)
                ctx.startActivity(
                    Intent(Intent.ACTION_VIEW)
                        .setDataAndType(uri, "application/vnd.android.package-archive")
                        .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_ACTIVITY_NEW_TASK),
                )
            }
            Result.Launched(rel.version, needsPermission)
        } catch (e: Exception) {
            Log.w(TAG, "update check failed", e)
            Result.Failed(e.message ?: e.javaClass.simpleName)
        }
    }

    private fun latestRelease(): Release? {
        val json = httpGet(API)
        val tag = Regex("\"tag_name\"\\s*:\\s*\"([^\"]+)\"").find(json)?.groupValues?.get(1) ?: return null
        val urls = Regex("\"browser_download_url\"\\s*:\\s*\"([^\"]+)\"").findAll(json).map { it.groupValues[1] }.toList()
        val apk = urls.firstOrNull { it.endsWith(".apk") } ?: return null
        return Release(tag.removePrefix("v"), apk, apk.substringAfterLast('/'), urls.firstOrNull { it.endsWith("SHA256SUMS.txt") })
    }

    private fun open(url: String): HttpURLConnection =
        (URL(url).openConnection() as HttpURLConnection).apply {
            connectTimeout = 10_000
            readTimeout = 30_000
            setRequestProperty("User-Agent", "maid-cafe-se-updater")
            setRequestProperty("Accept", "application/vnd.github+json")
        }

    private fun httpGet(url: String): String {
        val c = open(url)
        try {
            if (c.responseCode != 200) throw IllegalStateException("HTTP ${c.responseCode}")
            return c.inputStream.bufferedReader().readText()
        } finally {
            c.disconnect()
        }
    }

    private fun download(url: String, to: File) {
        val c = open(url)
        try {
            if (c.responseCode != 200) throw IllegalStateException("ダウンロード失敗 HTTP ${c.responseCode}")
            c.inputStream.use { i -> to.outputStream().use { o -> i.copyTo(o) } }
        } finally {
            c.disconnect()
        }
    }

    /** SHA256SUMS.txtがあれば照合する。食い違えばAPKを消して失敗にする。 */
    private fun verify(rel: Release, apk: File) {
        val sumsUrl = rel.sumsUrl ?: return
        val expected = httpGet(sumsUrl).lines()
            .firstOrNull { it.trim().endsWith(rel.apkName) }?.trim()?.substringBefore(' ')?.lowercase() ?: return
        val md = MessageDigest.getInstance("SHA-256")
        apk.inputStream().use { i ->
            val buf = ByteArray(64 * 1024)
            while (true) { val n = i.read(buf); if (n < 0) break; md.update(buf, 0, n) }
        }
        val actual = md.digest().joinToString("") { "%02x".format(it) }
        if (actual != expected) {
            apk.delete()
            throw IllegalStateException("APKのハッシュが一致しません")
        }
    }
}
