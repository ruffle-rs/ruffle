package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

import flash.net.FileReference;

var file = new FileReference();

try {
  trace("file.data: " + file.data)
} catch (e) {
  trace("file.data threw: " + e);
}

try {
  trace("file.name: " + file.name)
} catch (e) {
  trace("file.name threw: " + e);
}

try {
  trace("file.size: " + file.size)
} catch (e) {
  trace("file.size threw: " + e);
}

try {
  file.load();
} catch (e) {
  trace("file.load() threw: " + e);
}

/*
// Not yet implemented

try {
  trace("file.creationDate: " + file.creationDate)
} catch (e) {
  trace("file.creationDate threw: " + e);
}
try {
  trace("file.creator: " + file.creator)
} catch (e) {
  trace("file.creator threw: " + e);
}
try {
  trace("file.modificationDate: " + file.modificationDate)
} catch (e) {
  trace("file.modificationDate threw: " + e);
}

// XXX AIR only
try {
  trace("file.extension: " + file.extension)
} catch (e) {
  trace("file.extension threw: " + e);
}

try {
  trace("file.type: " + file.type)
} catch (e) {
  trace("file.type threw: " + e);
  trace("e.name: " + e.name);
}

*/
