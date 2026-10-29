# Update the Gmail app password used for admin-login e-mails on the VPS (maid-cafe.tokyo / open-english, and open-web-server).
#
# How to use:
#   1) Google Account -> Security -> 2-Step Verification (must be ON) -> App passwords -> create one (any name) -> copy the 16 characters.
#   2) Run:  powershell -ExecutionPolicy Bypass -File F:\maid-cafe-se\tools\set-smtp-password.ps1
#   3) Type the app password when asked (it is hidden). Spaces are ignored.
#
# The password is sent to the VPS over SSH (standard input) and written to the two environment files there.
# It is never printed, never put on a command line, and never saved on this PC.
# Requires the SSH host alias "conoha" (see ~/.ssh/config), the same one used for deployment.

$ErrorActionPreference = 'Stop'

$sec = Read-Host 'Gmail app password (16 characters; spaces are ignored)' -AsSecureString
$bstr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($sec)
try { $pw = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($bstr) } finally { [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr) }
$pw = ($pw -replace '\s', '')
if ($pw.Length -ne 16 -or $pw -notmatch '^[A-Za-z0-9]+$') {
    throw "An app password is 16 letters/digits (got $($pw.Length) characters). Nothing was changed."
}

$remote = @'
set -e
upd() {
  f="$1"; k="$2"
  if grep -q "^$k=" "$f"; then
    tmp=$(mktemp)
    awk -v k="$k" 'BEGIN{FS=OFS="="} $1==k{print k"="ENVIRON["PW"]; next} {print}' "$f" > "$tmp"
    cat "$tmp" > "$f"; rm -f "$tmp"
    echo "updated $k in $f"
  else
    echo "skipped: $k not found in $f"
  fi
}
upd /etc/open-english.env OPEN_ENGLISH_SMTP_PASSWORD
upd /etc/open-web-server/admin.env OPEN_WEB_SERVER_SMTP_PASSWORD
chmod 600 /etc/open-web-server/admin.env
systemctl restart open-english
sleep 6
systemctl is-active open-english
echo "open-web-server picks up the new value at its next restart."
'@

$payload = ($pw + "`n" + $remote) -replace "`r", ''
$payload | ssh conoha 'IFS= read -r PW; export PW; bash -s'
$pw = $null
Write-Host 'Done. Try the administrator login on https://maid-cafe.tokyo/ again.'
