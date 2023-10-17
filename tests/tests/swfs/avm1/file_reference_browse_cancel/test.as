class Main {
    static function print_file_ref_vars(fileRef) {
        trace("creationDate = " + fileRef.creationDate.toString());
        trace("name = " + fileRef.name);
        trace("modificationDate = " + fileRef.modificationDate.toString());
        trace("size = " + fileRef.size.toString());
        trace("type = " + fileRef.type);
    }
    
	static function main(root) {
        var fileRef = new flash.net.FileReference();
        print_file_ref_vars(fileRef);
    
		var listener = new Object();
        listener.onSelect = function(file) {
            trace("Opened " + file.name);
            print_file_ref_vars(fileRef);
        }
        listener.onCancel = function(file) {
            trace("User cancelled");
            print_file_ref_vars(fileRef);
        }
        
        fileRef.addListener(listener);
        fileRef.browse();
	}
}
