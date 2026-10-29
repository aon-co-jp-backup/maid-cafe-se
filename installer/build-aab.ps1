# maid-cafe-se 署名つきAndroid App Bundle(.aab)のビルド。 / Build the signed Android App Bundle (.aab).
#
# 使い方 / Usage:  powershell -ExecutionPolicy Bypass -File installer\build-aab.ps1
# 鍵情報ファイルの形式は build-release.ps1 と同じ(リポジトリの外。コミットしない。値は出力しない)。
# 出力: installer\dist\maid-cafe-se_<version>_android.aab、SHA256SUMS.txt にその行を追加(APKの行は残す)。
param(
    [string]$KeystoreInfo = "F:\maid-cafe-se-keystore.txt",
    [string]$JavaHome = "C:\Program Files\Java\jdk-17"
)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot

if (-not (Test-Path $KeystoreInfo)) { throw "鍵情報ファイルがありません / key info file not found: $KeystoreInfo" }
$info = @{}
Get-Content $KeystoreInfo | ForEach-Object { if ($_ -match '^([A-Z_]+)=(.*)$') { $info[$Matches[1]] = $Matches[2] } }
foreach ($k in 'KEYSTORE_FILE', 'KEYSTORE_PASSWORD', 'KEY_ALIAS', 'KEY_PASSWORD') {
    if (-not $info[$k]) { throw "鍵情報に $k がありません / missing $k in $KeystoreInfo" }
}
if (-not (Test-Path $info['KEYSTORE_FILE'])) { throw "keystoreがありません / keystore not found: $($info['KEYSTORE_FILE'])" }

$env:MAID_CAFE_SE_KEYSTORE_FILE = $info['KEYSTORE_FILE']
$env:MAID_CAFE_SE_KEYSTORE_PASSWORD = $info['KEYSTORE_PASSWORD']
$env:MAID_CAFE_SE_KEY_ALIAS = $info['KEY_ALIAS']
$env:MAID_CAFE_SE_KEY_PASSWORD = $info['KEY_PASSWORD']
if (Test-Path $JavaHome) { $env:JAVA_HOME = $JavaHome }

$gradle = Get-Content (Join-Path $root 'app\build.gradle.kts') -Raw
if ($gradle -notmatch 'versionName\s*=\s*"([^"]+)"') { throw "versionNameが読めません / cannot read versionName" }
$version = $Matches[1]

Push-Location $root
try {
    & "$root\gradlew.bat" ':app:bundleRelease' '--console=plain'
    if ($LASTEXITCODE -ne 0) { throw "ビルド失敗 / build failed" }
} finally { Pop-Location }

$built = Join-Path $root 'app\build\outputs\bundle\release\app-release.aab'
if (-not (Test-Path $built)) { throw "aabができていません / aab not produced: $built" }
$dist = Join-Path $PSScriptRoot 'dist'
New-Item -ItemType Directory -Force $dist | Out-Null
$out = Join-Path $dist "maid-cafe-se_${version}_android.aab"
Copy-Item $built $out -Force

# 署名検証: jarsigner(aabはJAR署名)。値(パスワード)は出さない。
$jarsigner = Join-Path $env:JAVA_HOME 'bin\jarsigner.exe'
& $jarsigner -verify $out
if ($LASTEXITCODE -ne 0) { throw "署名の検証に失敗 / signature verification failed" }

$hash = (Get-FileHash $out -Algorithm SHA256).Hash.ToLower()
$sums = Join-Path $dist 'SHA256SUMS.txt'
$name = Split-Path -Leaf $out
$keep = if (Test-Path $sums) { Get-Content $sums | Where-Object { $_ -notmatch '_android\.aab' } } else { @() }
@($keep) + "$hash  $name" | Set-Content $sums -Encoding ascii
Write-Host "OK: $out"
Write-Host "SHA256: $hash"
