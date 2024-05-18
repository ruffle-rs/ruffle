// compiled with mxmlc


namespace ruffle = "__ruffle__";

function print(text) {
    trace(text);
    try {
        trace(text.ruffle::isDependent());
    } catch(e) {
        trace();
    }
}

function test(){
    trace("--- substrings ---")
    var text = "abcd";
    print(text.substr(0));
    print(text.substr(1));
    print(text.substr(2));
    print(text.substr(3));
    print(text.substr(4));

    trace("--- pure concat ---")
    text = "";
    for (var i = 0; i < 40; i += 5) {
        text += "aaaaa";
        print(text);
    }
    trace("--- pure concat, wide ---")
    text = "ą";
    for (var i = 0; i < 40; i += 5) {
        text += "ąaaaa";
        print(text);
    }

    trace("--- wide concat, then substring ---")
    text = "ą1234";
    text += "ę"
    text += "ą"
    print(text);
    text = text.substring(1);
    print(text);

    trace("--- substring, then dependent concat ---")
    text = "1234";
    text += "5";
    print(text);
    print(text.substring(1));
    print(text.substring(1) + "6");
    print(text.substring(1) + "7");

    trace("--- substring, then non-dependent concat ---")
    text = "1234";
    text += "5";
    print(text);
    print(text.substring(0, 2));
    print(text.substring(0, 2) + "6");

    trace("--- wide substring, then dependent concat ---")
    text = "ąąąą";
    text += "5ą";
    print(text);
    print(text.substring(1));
    print(text.substring(1) + "6");
    print(text.substring(1) + "6ą");
    print(text.substring(1) + "7ą");

    trace("--- self-append ---")
    text = "1234";
    text += "5";
    print(text);
    print(text + text);
    print(text + text);
}
test();

package {

    import flash.display.MovieClip;
    import flash.text.TextField;

    public class Test extends MovieClip {
        public function Test(){
        }
    }
}
