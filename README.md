# AutoTyper

把文字檔的內容，模擬鍵盤逐字輸入到目前取得焦點的視窗。

## 下載方式

到 [Releases](https://github.com/flyflyk/Auto-Typer/releases/latest) 下載 `AutoTyper-Setup.exe` 並執行。安裝不需要系統管理員權限，完成後開始功能表和桌面會出現 `AutoTyper`，之後有新版本時程式會提示更新。要移除時，到「設定 → 應用程式」解除安裝。

不想安裝的話，也可以下載 `auto-typer.exe` 直接執行；程式會提示一次是否要安裝，按「Not now」之後就不再詢問。

第一次執行時，Windows 可能顯示 SmartScreen 警告，按「其他資訊」→「仍要執行」。

## 使用方式

1. 開啟程式，把檔案拖進視窗，或按 Choose file 選擇檔案（需為 UTF-8 編碼）。也可以拖入資料夾或按 Choose folder，見下方「複製資料夾」。
2. 把輸入法切換成英文。
3. 視需要調整設定，按 Start。檔案區會預覽實際要輸入的內容，打字時會即時標出進度。
4. 5 秒內點進要輸入的目標視窗，程式會開始打字。
5. 任何時候按 Esc 可立即停止。

## 設定說明

| 設定 | 說明 |
|---|---|
| Delay per character | 每個字元之間的延遲。遠端桌面掉字時調高，例如 30 ms 以上。可拖曳滑桿，或點右側數字直接輸入（最多 5000 ms）。 |
| Shift gap | 大寫字母和符號的 Shift 與按鍵之間的停頓。經過多層遠端連線（例如 AnyDesk 再接遠端桌面）時大小寫會打錯的話，從 20 到 50 ms 開始調高；0 表示一起送出。只在 Keystrokes 模式有效。可直接輸入數字（最多 1000 ms）。 |
| Input method：Keystrokes | 模擬實體按鍵，相容遠端桌面、VM、終端機；非 ASCII 字元自動改用 Unicode。 |
| Input method：Unicode | 直接送出 Unicode 字元，支援中文，但部分遠端桌面或 VM 不支援。 |
| Strip leading indentation | 移除每行行首空白，避免編輯器自動縮排造成縮排累加。 |

## 複製資料夾

選擇資料夾後，程式不會直接輸入檔案內容，而是輸入一段 Shell 指令，在目標終端機的目前目錄重建整個資料夾（含子資料夾、空資料夾與所有檔案）。

1. 在 Target shell 選擇 Bash（Linux、macOS）或 PowerShell（Windows）。
2. 在目標電腦開啟對應的終端機，`cd` 到要放置資料夾的位置。
3. 按 Start，5 秒內點進該終端機。

- 純文字檔照原樣輸入；其他檔案（圖片、執行檔等）以 base64 輸入後還原，內容逐位元組相同，但輸入時間較長。
- `.git` 資料夾不會複製；檔案總大小上限 5 MB。
- Bash 的檔案一律為 LF 換行；PowerShell 保留原本的 LF 或 CRLF，並存成無 BOM 的 UTF-8。
- 輸入期間會暫時關閉 Tab 自動完成（Bash 也會關閉 `!` 歷史展開），結束時恢復。
