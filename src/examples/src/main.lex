pub fn main() -> void {
    let webview = WebView::create({
        title: "My WebView App",
        width: 800,
        height: 600,
        html: "<html><body style='font-family: Arial; text-align: center; padding: 50px;'><h1>Hello from WebView!</h1><button onclick='alert(\"Hello!\")' style='padding: 10px 20px;'>Click Me</button></body></html>"
    });
    
    webview.show();
}
