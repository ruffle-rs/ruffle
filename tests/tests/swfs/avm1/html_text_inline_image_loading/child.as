Stage.scaleMode = "noScale";
Stage.align = "TL";

_root.createEmptyMovieClip("box", 1);
var box:MovieClip = _root["box"];

box.beginFill(0xFF0000);
box.moveTo(0, 0);
box.lineTo(100, 0);
box.lineTo(100, 50);
box.lineTo(0, 50);
box.lineTo(0, 0);
box.endFill();

box.lineStyle(4, 0x000000);
box.moveTo(0, 0);
box.lineTo(100, 50);
box.moveTo(100, 0);
box.lineTo(0, 50);

_root.createTextField("label", 2, 5, 15, 90, 20);
var childLabel:TextField = _root["label"];
childLabel.text = "CHILD";

trace("CHILD_SWF_LOADED");
trace("CHILD_STAGE=100x50");
