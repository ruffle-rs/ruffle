// compiled with mxmlc

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {

        }
    }
}


trace("// string replacements")
trace("a a a".replace("a", ""));
trace("a a a".replace("a", "b"));
trace("aaaa".replace("aa", "a"));
trace("a a a".replace("", "x"));

trace("// regex")
trace("  123".replace(/123/g, "x"));
trace("123  ".replace(/123/g, "x"));
trace("  123  ".replace(/123/g, "x"));

trace("123  123".replace(/ +/g, "x"));
trace("123  123".replace(/\d+/g, "x"));
trace("123  123".replace(/.*/g, "x"));

trace("// empty regex")
trace("aaa".replace(new RegExp("", "g"), "x"));

trace("// lastIndex should not be modified")
var regex = /a/g;
regex.lastIndex = 1;
trace("aaaa".replace(regex, "x"));
trace(regex.lastIndex);

trace("// $ with non-special successor char")
trace("abaa".replace(/b/,"$k"))

trace("// $& ")
trace("abaa".replace(/b/,"|$&|"))

trace("// $` ")
trace("axbfg".replace(/b/,"$`"))

trace("// $' ")
trace("axbfg".replace(/b/,"$'"))


trace("// $1")
trace("abc".replace(/(b)/, "<$1>"))

trace("// not enough groups")
trace("abc".replace(/(b)/, "<$2>"))

trace("// two-digit capture group number, but not enough groups")
trace("abc".replace(/(b)/, "<$10>"))

trace("// Two-digit capture group number")
var r=RegExp("(.)(.)(.)(.)(.)(.)(.)(.)(.)(.)")
trace("abbbbbbbbb#bbc".replace(r, "<$10>"))



