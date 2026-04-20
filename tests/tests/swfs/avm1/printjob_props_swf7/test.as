var o = new PrintJob();
trace(PrintJob);
trace(typeof PrintJob);
trace(PrintJob.prototype);
trace(typeof PrintJob.prototype);

trace(o);
trace(typeof o);

trace("Enumerated");
for (var p in o) {
    trace(p);
}

trace("Enumerated prototype");
for (var p in PrintJob.prototype) {
    trace(p);
}

trace("Props");
trace(o.start);
trace(o.addPage);
trace(o.send);
trace(o.paperHeight);
trace(o.paperWidth);
trace(o.pageHeight);
trace(o.pageWidth);
trace(o.orientation);

trace("After set");
o.start = "test";
o.addPage = "test";
o.send = "test";
o.paperHeight = "test";
o.paperWidth = "test";
o.pageHeight = "test";
o.pageWidth = "test";
o.orientation = "test";
trace(o.start);
trace(o.addPage);
trace(o.send);
trace(o.paperHeight);
trace(o.paperWidth);
trace(o.pageHeight);
trace(o.pageWidth);
trace(o.orientation);

trace("After delete");
delete o.start;
delete o.addPage;
delete o.send;
delete o.paperHeight;
delete o.paperWidth;
delete o.pageHeight;
delete o.pageWidth;
delete o.orientation;
trace(o.start);
trace(o.addPage);
trace(o.send);
trace(o.paperHeight);
trace(o.paperWidth);
trace(o.pageHeight);
trace(o.pageWidth);
trace(o.orientation);
