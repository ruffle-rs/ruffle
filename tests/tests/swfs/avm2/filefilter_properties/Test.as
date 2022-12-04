import flash.net.FileFilter;

package {
    public class Test {}
}

function print_description() {
    var fileFilter: FileFilter = new FileFilter("Images", "*.jpg;*.gif;*.png");
    trace(fileFilter.description);
    trace(fileFilter.extension);
    trace(fileFilter.macType);
}

print_description();
