message-cant-embed =
    目前Ruffle沒辦法執行嵌入式Flash。
    你可以在新分頁中開啟來解決這個問題。
panic-title = 完蛋，出問題了 :(
more-info = 更多資訊
run-anyway = 直接執行
continue = 繼續
report-bug = 回報BUG
update-ruffle = 更新Ruffle
ruffle-demo = 網頁展示
ruffle-desktop = 桌面應用程式
ruffle-wiki = 查看Ruffle Wiki
enable-hardware-acceleration =
    看起來你的硬體加速沒有開啟，雖然Ruffle還可以執行，但是你會感覺到會很慢。
    你可以在下方連結找到如何開啟硬體加速。
enable-hardware-acceleration-link = FAQ - Chrome硬體加速
view-error-details = 檢視錯誤詳細資料
open-in-new-tab = 開啟新增分頁
click-to-unmute = 點擊以取消靜音
clipboard-message-title = 在Ruffle執行複製、貼上。
clipboard-message-description =
    { $variant ->
       *[unsupported] 你的瀏覽器不支援完整剪貼簿使用,
        [access-denied] 剪貼簿使用權被拒絕訪問,
    } 你可以直接使用快捷鍵:
clipboard-message-copy = { " " } 複製
clipboard-message-cut = { " " } 剪下
clipboard-message-paste = { " " } 貼上
error-file-protocol =
    看起來你想要用Ruffle來執行"file:"的協議。
    因為瀏覽器禁了很多功能以資安的理由來講。
    我們建議你建立本地伺服器或著直接使用網頁展示或桌面應用程式。
error-javascript-config =
    目前Ruffle遇到不正確的JavaScript配置。
    如果你是伺服器管理員，我們建議你檢查哪個環節出錯。
    或著你可以查詢Ruffle wiki得到需求幫助。
error-wasm-not-found =
    目前Ruffle找不到".wasm"檔案。
    如果你是伺服器管理員，確保檔案是否放對位置。
    如果還是有問題的話，你要用"publicPath"來設定: 或著查詢Ruffle wiki得到需求幫助。
error-wasm-mime-type =
    目前Ruffle初始化時遇到重大問題。
    這網頁伺服器並沒有服務".wasm"檔案或正確的網際網路媒體類型。
    如果你是伺服器管理員，請查詢Ruffle wiki得到需求幫助。
error-invalid-swf =
    目前Ruffle無法讀取Flash的SWF檔案。
    很有可能要讀取的檔案並一個不是有效的SWF。
error-swf-fetch =
    目前Ruffle無法讀取Flash的SWF檔案。
    很有可能要讀取的檔案不存在，所以Ruffle讀不到東西。
    請嘗試溝通伺服器管理員得到需求幫助。
error-swf-cors =
    目前Ruffle無法讀取Flash的SWF檔案。
    看起來是使用權被跨來源資源共用機制被擋到了。
    如果你是伺服器管理員，請查詢Ruffle wiki得到需求幫助。
error-wasm-cors =
    目前Ruffle無法讀取".wasm"檔案。
    看起來是使用權被跨來源資源共用機制被擋到了。
    如果你是伺服器管理員，請查詢Ruffle wiki得到需求幫助。
error-wasm-invalid =
    目前Ruffle初始化時遇到重大問題。
    看起來這網頁有缺失檔案導致Ruffle無法運行。
    如果你是伺服器管理員，請查詢Ruffle wiki得到需求幫助。
error-wasm-download =
    目前Ruffle初始化時遇到重大問題。
    這可以你自己解決，你只要重新整理就好了。
    否則，請嘗試溝通伺服器管理員得到需求幫助。
error-wasm-disabled-on-edge =
    目前Ruffle無法讀取".wasm"檔案。
    要修正的話，打開你的瀏覽器設定，點選"隱私權、搜尋與服務"，把"防止追蹤"給關掉。
    這樣一來你的瀏覽器會讀取需要的".wasm"檔案。
    如果問題一直還在的話，你必須要換瀏覽器了。
error-javascript-conflict =
    目前Ruffle初始化時遇到重大問題。
    看起來這網頁使用的JavaScript會跟Ruffle起衝突。
    如果你是伺服器管理員，我們建議你開個空白頁來測試。
error-javascript-conflict-outdated = 你也可以上傳最新版的Ruffle，說不定你要說的的問題已經不見了(現在使用的版本已經過時: { $buildDate })。
error-csp-conflict =
    目前Ruffle初始化時遇到重大問題。
    這網頁伺服器被跨來源資源共用機制禁止讀取".wasm"檔案。
    如果你是伺服器管理員，請查詢Ruffle wiki得到需求幫助。
error-unknown =
    目前Ruffle初始化要讀取Flash內容時遇到重大問題
    { $outdated ->
        [true] 如果你是伺服器管理員， 請上傳最新版的Ruffle(現在使用的版本已經過時: { $buildDate }).
       *[false] 這不應該發生的，我們也很高興你告知bug!
    }
