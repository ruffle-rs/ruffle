// compiled with mxmlc



var arr = [1, 2];
arr[50] = 6;
arr[100] = 10;
arr[500] = 11;

trace(arr[0]);
trace(arr[50]);
trace(arr[100]);
trace(arr[500]);
trace(arr[1000]);
trace(arr.length);
trace("// delete")
delete arr[50];
trace(arr[50]);
trace(arr[100]);
trace("// push")
arr.push(12);
trace(arr.length);
trace(arr[501]);
trace(arr.pop());
trace(arr.length);
trace(arr[500]);
trace("// for")
for (var i in arr) {
    trace(i);
}
trace("// for each")
for each (var i in arr) {
    trace(i);
}
trace("// shift");
trace(arr.shift());
trace(arr.length);
trace(arr[0]);
trace(arr[99]);
trace(arr[499]);
trace("// shift")
arr.unshift(1);
trace(arr.length);
trace(arr[0]);
trace(arr[100]);
trace(arr[500]);
trace("// removeAt");
trace(arr.removeAt(150));
trace(arr.length);
trace(arr[499]);
trace(arr[500]);


package {
    import flash.display.MovieClip;
    import flash.text.TextField;

    public class Test extends MovieClip {
        public function Test(){
        }
    }
}

