import core.webview.WebView;
import core.webview.MessageBox;

pub fn main() -> void {
    let webview = WebView::create({
        title: "My WebView App",
        width: 800,
        height: 600,
        html: "<html><body><h1>Hello from WebView!</h1><button onclick='alert(\"Hello!\")'>Click Me</button></body></html>"
    });
    
    webview.show();
}
