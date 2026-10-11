# Auto Typer

把文字檔的內容，模擬鍵盤逐字輸入到目前取得焦點的視窗。

## 下載方式

到 [Releases](https://github.com/OWNER/auto-typer/releases/latest) 下載 `auto-typer.exe`，不需要安裝，直接執行即可。

第一次執行時，Windows 可能顯示 SmartScreen 警告，按「其他資訊」→「仍要執行」。

## 使用方式

1. 開啟程式，把檔案拖進視窗，或按 Browse 選擇檔案（需為 UTF-8 編碼）。
2. 把輸入法切換成英文。
3. 視需要調整設定，按 Start。
4. 5 秒內點進要輸入的目標視窗，程式會開始打字。
5. 任何時候按 Esc 可立即停止。

## 設定說明

| 設定 | 說明 |
|---|---|
| Delay per character | 每個字元之間的延遲。遠端桌面掉字時調高，例如 30 ms 以上。 |
| Shift gap | 大寫字母和符號的 Shift 與按鍵之間的停頓。經過多層遠端連線（例如 AnyDesk 再接遠端桌面）時大小寫會打錯的話，從 20 到 50 ms 開始調高；0 表示一起送出。只在 Keystrokes 模式有效。 |
| Keystrokes | 模擬實體按鍵，相容遠端桌面、VM、終端機；非 ASCII 字元自動改用 Unicode。 |
| Unicode | 直接送出 Unicode 字元，支援中文，但部分遠端桌面或 VM 不支援。 |
| Strip leading indentation | 移除每行行首空白，避免編輯器自動縮排造成縮排累加。 |
