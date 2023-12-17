const { St, Clutter } = imports.gi;
const Main = imports.ui.main;

let panelButton;

const GLib = imports.gi.GLib;

function init() {
    log("Initializing the extension");
    // Create a Button with initial text
    panelButton = new St.Bin({
        style_class: "panel-button",
    });
    let panelButtonText = new St.Label({
        text: "Loading...",
        y_align: Clutter.ActorAlign.CENTER,
    });
    panelButton.set_child(panelButtonText);


    let decoder = new TextDecoder();
        
    // Read and update the text every second
    GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 5, () => {
        let [success, contents] = GLib.file_get_contents("/tmp/typer-timer/banner");
        contents_str = decoder.decode(contents)
        if (success) {
            panelButtonText.set_text(contents_str);
        }    
        return true;
    });
}

function enable() {
    // Add the button to the panel
    Main.panel._rightBox.insert_child_at_index(panelButton, 0);
}

function disable() {
    // Remove the added button from panel
    Main.panel._rightBox.remove_child(panelButton);
}
