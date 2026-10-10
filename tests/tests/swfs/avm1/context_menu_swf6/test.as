// ContextMenu and ContextMenuItem methods are hidden from SWF 6.

function keys(obj) {
    var result = "";
    for (var key in obj) {
        result += key + " ";
    }
    return result;
}

trace("// typeof ContextMenu.prototype.copy");
trace(typeof ContextMenu.prototype.copy);
trace("");

trace("// typeof ContextMenu.prototype.hideBuiltInItems");
trace(typeof ContextMenu.prototype.hideBuiltInItems);
trace("");

trace("// typeof ContextMenuItem.prototype.copy");
trace(typeof ContextMenuItem.prototype.copy);
trace("");

trace("// new ContextMenu()");
trace(keys(new ContextMenu()));
trace("");

trace("// new ContextMenuItem('Caption')");
trace(keys(new ContextMenuItem("Caption")));
trace("");
