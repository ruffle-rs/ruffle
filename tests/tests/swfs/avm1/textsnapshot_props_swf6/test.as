var o = new TextSnapshot();

trace(TextSnapshot);
trace(typeof TextSnapshot);
trace(TextSnapshot.prototype);
trace(typeof TextSnapshot.prototype);

trace(o);
trace(typeof o);

trace("Enumerated");
for (var p in o) {
    trace(p);
}

trace("Enumerated prototype");
for (var p in TextSnapshot.prototype) {
    trace(p);
}

trace("Props");
trace(o.getTextRunInfo);
trace(o.setSelectColor);
trace(o.findText);
trace(o.hitTestTextNearPos);
trace(o.getSelectedText);
trace(o.getText);
trace(o.getSelected);
trace(o.setSelected);
trace(o.getCount);

trace("After set");
o.getTextRunInfo = "test";
o.setSelectColor = "test";
o.findText = "test";
o.hitTestTextNearPos = "test";
o.getSelectedText = "test";
o.getText = "test";
o.getSelected = "test";
o.setSelected = "test";
o.getCount = "test";
trace(o.getTextRunInfo);
trace(o.setSelectColor);
trace(o.findText);
trace(o.hitTestTextNearPos);
trace(o.getSelectedText);
trace(o.getText);
trace(o.getSelected);
trace(o.setSelected);
trace(o.getCount);

trace("After delete");
delete o.getTextRunInfo;
delete o.setSelectColor;
delete o.findText;
delete o.hitTestTextNearPos;
delete o.getSelectedText;
delete o.getText;
delete o.getSelected;
delete o.setSelected;
delete o.getCount;
trace(o.getTextRunInfo);
trace(o.setSelectColor);
trace(o.findText);
trace(o.hitTestTextNearPos);
trace(o.getSelectedText);
trace(o.getText);
trace(o.getSelected);
trace(o.setSelected);
trace(o.getCount);
