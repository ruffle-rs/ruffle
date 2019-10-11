var document = fl.getDocumentDOM();
if(document) {
	var element = document.getTimeline().layers[0].frames[0].elements[0]; 
	if(element) {
		element.setPersistentData("test", "string", "Testing!");
		element.setPublishPersistentData("test", "_EMBED_SWF_", true);
		document.setPublishDocumentData("_EMBED_SWF_", true); 
		alert("Added PlaceObject4 data for clip.");
	} else {
		alert("Please select a symbol in the library.");
	}
}