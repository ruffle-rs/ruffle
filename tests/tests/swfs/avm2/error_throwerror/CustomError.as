package {
public class CustomError extends Error {
    public function CustomError(...rest) {
        trace("CustomError instantiated");
        for each (var arg in rest) {
            trace(arg);
            trace(arg.constructor);
        }
    }
}
}
