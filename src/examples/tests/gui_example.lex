// =====================================================
// Lexicon GUI Example - Simple Window
// =====================================================

import core.gui.Window;
import core.gui.Button;
import core.gui.Label;
import core.gui.TextField;
import core.gui.TextArea;
import core.gui.Checkbox;
import core.gui.ComboBox;
import core.gui.ListView;
import core.gui.MenuBar;
import core.gui.Menu;
import core.gui.MenuItem;
import core.gui.MessageBox;
import core.gui.Color;

fn simpleWindowExample() -> void {
    let window = Window::create({ title: "My First App", width: 800, height: 600 });
    window.setBackground(Color::fromRgb(240, 240, 245));
    
    let titleLabel = Label::create("Welcome to Lexicon GUI!");
    window.add(titleLabel);
    
    let clickButton = Button::create("Click Me!");
    clickButton.onClick(fn(event) => {
        Console.writeLine("Button clicked!");
        MessageBox::show("Hello!", "You clicked the button!");
    });
    window.add(clickButton);
    
    window.show();
}

fn formExample() -> void {
    let window = Window::create({ title: "Registration Form", width: 500, height: 400 });
    
    let header = Label::create("Create Account");
    window.add(header);
    
    let usernameField = TextField::create();
    usernameField.setPlaceholder("Enter username");
    window.add(usernameField);
    
    let emailField = TextField::create();
    emailField.setPlaceholder("Enter email");
    window.add(emailField);
    
    let passwordField = TextField::create();
    passwordField.setPassword(true);
    passwordField.setPlaceholder("Enter password");
    window.add(passwordField);
    
    let rememberCheck = Checkbox::create("Remember me");
    window.add(rememberCheck);
    
    let submitBtn = Button::create("Register");
    submitBtn.onClick(fn(event) => {
        Console.writeLine("Registration submitted!");
        MessageBox::show("Success!", "Account created!");
        window.close();
    });
    window.add(submitBtn);
    
    window.show();
}

fn menuExample() -> void {
    let window = Window::create({ title: "Menu Demo", width: 600, height: 400 });
    
    let menuBar = MenuBar::new();
    
    let fileMenu = Menu::new("File");
    fileMenu.add(MenuItem::new("New", "Ctrl+N", fn() => {
        Console.writeLine("New file");
    }));
    fileMenu.add(MenuItem::new("Open", "Ctrl+O", fn() => {
        Console.writeLine("Open file");
    }));
    fileMenu.add(MenuItem::new("Exit", fn() => {
        window.close();
    }));
    menuBar.add(fileMenu);
    
    let helpMenu = Menu::new("Help");
    helpMenu.add(MenuItem::new("About", fn() => {
        MessageBox::show("About", "Lexicon GUI Demo v1.0");
    }));
    menuBar.add(helpMenu);
    
    window.setMenuBar(menuBar);
    window.show();
}

pub fn main() -> void {
    simpleWindowExample();
}
