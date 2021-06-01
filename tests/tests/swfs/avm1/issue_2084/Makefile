CLIPS=Main.as Column.as
IMAGES=icon.png

ALLSRC=$(CLIPS) $(IMAGES) test.xml

test.swf: test.xml classes.swf $(IMAGES)
	swfmill simple test.xml test.swf

classes.swf: $(CLIPS)
	rm -f classes.swf
	mtasc -swf classes.swf -header 1:1:1:000000 -version 7 -strict -infer $(CLIPS)

clean:
	rm -f classes.swf
