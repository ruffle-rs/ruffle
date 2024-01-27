package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

import flash.net.FileReference;

function dump(file) {
    var properties = ["creationDate", "creator", "data", /* AIR */ "extension", "modificationDate", "name", "size", "type"];

    for each (var property in properties) {
        try {
            trace("file['" + property + "']: " + file[property]);
        } catch (e) {
            trace("file['" + property + "'] throw: " + e);
        }
    }
}

var file = new FileReference();
dump(file);
