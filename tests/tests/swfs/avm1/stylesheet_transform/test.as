function traceTextFormat(tf) {
    trace("  tf=" + tf +
        ", size=" + tf.size +
        ", blockIndent=" + tf.blockIndent +
        ", font=" + tf.font +
        ", align=" + tf.align +
        ", leading=" + tf.leading +
        ", display=" + tf.display +
        ", kerning=" + tf.kerning +
        ", letterSpacing=" + tf.letterSpacing +
        ", leftMargin=" + tf.leftMargin +
        ", rightMargin=" + tf.rightMargin +
        ", color=" + tf.color +
        ", bold=" + tf.bold +
        ", italic=" + tf.italic +
        ", bullet=" + tf.bullet +
        ", underline=" + tf.underline);
}

function runTest(name, object) {
    var style = new TextField.StyleSheet();
    var tf = style.transform(object);
    trace("Testing " + name + ":");
    traceTextFormat(tf);
}

function runInitialTests() {
    trace("Calling transform with no args:");
    var style = new TextField.StyleSheet();
    var tf = style.transform();
    trace("  " + tf);
}

function runTestNumerical(name, objectConstructor) {
    runTest(name + " null", objectConstructor(null));
    runTest(name + " undefined", objectConstructor(undefined));
    runTest(name + " object", objectConstructor(new Object()));
    runTest(name + " true", objectConstructor(true));
    runTest(name + " false", objectConstructor(false));
    runTest(name + " number", objectConstructor(5));
    runTest(name + " number negative", objectConstructor(-5));
    runTest(name + " number 0", objectConstructor(0));
    runTest(name + " number 0.5", objectConstructor(0.5));
    runTest(name + " number 0.9", objectConstructor(0.9));
    runTest(name + " number 1.1", objectConstructor(1.1));
    runTest(name + " number 1.5", objectConstructor(1.5));
    runTest(name + " number 1.9", objectConstructor(1.9));
    runTest(name + " number -0.5", objectConstructor(-0.5));
    runTest(name + " number -0.9", objectConstructor(-0.9));
    runTest(name + " number -1.1", objectConstructor(-1.1));
    runTest(name + " number -1.5", objectConstructor(-1.5));
    runTest(name + " number -1.9", objectConstructor(-1.9));
    runTest(name + " number 255", objectConstructor(255));
    runTest(name + " number 256", objectConstructor(256));
    runTest(name + " number -255", objectConstructor(-255));
    runTest(name + " number -256", objectConstructor(-256));
    runTest(name + " number 65535", objectConstructor(65535));
    runTest(name + " number 65536", objectConstructor(65536));
    runTest(name + " number -65535", objectConstructor(-65535));
    runTest(name + " number -65536", objectConstructor(-65536));
    runTest(name + " string", objectConstructor('x'));
    runTest(name + " string number", objectConstructor('10'));
    runTest(name + " string empty", objectConstructor(''));
    runTest(name + " string space 1", objectConstructor(' 10'));
    runTest(name + " string space 2", objectConstructor('10 '));
    runTest(name + " string space 3", objectConstructor(' 10 '));
    runTest(name + " string space 4", objectConstructor('\t\n10\t\n'));
    runTest(name + " string negative", objectConstructor('-10'));
    runTest(name + " string negative space 1", objectConstructor(' -10'));
    runTest(name + " string negative space 2", objectConstructor('-10 '));
    runTest(name + " string negative space 3", objectConstructor(' -10 '));
    runTest(name + " string negative space 4", objectConstructor('- 10'));
    runTest(name + " string prefix 1", objectConstructor('prefix 10'));
    runTest(name + " string prefix 2", objectConstructor('prefix10'));
    runTest(name + " string suffix 1", objectConstructor('10 suffix'));
    runTest(name + " string suffix 2", objectConstructor('10suffix'));
}

runInitialTests();

runTest("empty", new Object());
runTest("number", 4.4);
runTest("string", "x");
runTest("null", null);
runTest("undefined", undefined);
runTest("movie clip", _root);

runTest("color number", {color:4});
runTest("color hex 1", {color:'#faFAfa'});
runTest("color hex 2", {color:' #faFAfa '});
runTest("color hex 3", {color:'#456'});
runTest("color hex 4", {color:'#faFAfa '});
runTest("color hex 5", {color:' #faFAfa'});
runTest("color hex 6", {color:'faFAfa'});

runTest("display inline 1", {display:'inline'});
runTest("display inline 2", {display:'Inline'});
runTest("display block", {display:'block'});
runTest("display none", {display:'none'});
runTest("display unknown", {display:'unknown'});

runTest("fontFamily null", {fontFamily:null});
runTest("fontFamily undefined", {fontFamily:undefined});
runTest("fontFamily true", {fontFamily:true});
runTest("fontFamily false", {fontFamily:false});
runTest("fontFamily basic", {fontFamily:'Font'});
runTest("fontFamily number", {fontFamily:5});
runTest("fontFamily number 0", {fontFamily:0});
runTest("fontFamily object", {fontFamily:new Object()});
runTest("fontFamily bool", {fontFamily:true});
runTest("fontFamily whitespace", {fontFamily:'  Font  '});
runTest("fontFamily comma separated list", {fontFamily:'  Font1  ,   Font2 ,, Font3 '});
runTest("fontFamily spaces in name", {fontFamily:'  Font 1  ,   Font  2  Fo     nt3 '});

runTest("fontSize string px", {fontSize:'42px'});
runTest("fontSize string pt", {fontSize:'42pt'});
runTestNumerical("fontSize", function(n) { return {fontSize:n}; });

runTest("fontStyle normal", {fontStyle:'normal'});
runTest("fontStyle italic", {fontStyle:'italic'});
runTest("fontStyle italic case", {fontStyle:'Italic'});
runTest("fontStyle spaces 1", {fontStyle:' italic'});
runTest("fontStyle spaces 2", {fontStyle:'italic '});
runTest("fontStyle spaces 3", {fontStyle:' italic '});
runTest("fontStyle unknown", {fontStyle:'unknown'});
runTest("fontStyle number", {fontStyle:1});

runTest("fontWeight normal", {fontWeight:'normal'});
runTest("fontWeight bold", {fontWeight:'bold'});
runTest("fontWeight bold case", {fontWeight:'Bold'});
runTest("fontWeight spaces 1", {fontWeight:' bold'});
runTest("fontWeight spaces 2", {fontWeight:'bold '});
runTest("fontWeight spaces 3", {fontWeight:' bold '});
runTest("fontWeight unknown", {fontWeight:'unknown'});
runTest("fontWeight number", {fontWeight:1});

runTest("kerning true", {kerning:true});
runTest("kerning false", {kerning:false});
runTest("kerning string false", {kerning:'false'});
runTest("kerning string true", {kerning:'true'});
runTest("kerning string unknown", {kerning:'unknown'});
runTest("kerning string empty", {kerning:''});
runTest("kerning number 0", {kerning:0});
runTest("kerning number 0.5", {kerning:0.5});
runTest("kerning number 1", {kerning:1});
runTest("kerning number 2", {kerning:2});
runTest("kerning number -1", {kerning:-1});
runTest("kerning object", {kerning:new Object()});
runTestNumerical("kerning", function(n) { return {kerning:n}; });

runTestNumerical("leading", function(n) { return {leading:n}; });
runTestNumerical("letterSpacing", function(n) { return {letterSpacing:n}; });
runTestNumerical("marginLeft", function(n) { return {marginLeft:n}; });
runTestNumerical("marginRight", function(n) { return {marginRight:n}; });

runTest("textAlign left", {textAlign:'left'});
runTest("textAlign center", {textAlign:'center'});
runTest("textAlign right", {textAlign:'right'});
runTest("textAlign justify", {textAlign:'justify'});
runTest("textAlign unknown", {textAlign:'unknown'});
runTest("textAlign center case", {textAlign:'Center'});
runTest("textAlign center case 2", {textAlign:'CENTER'});
runTest("textAlign center case 3", {textAlign:'CenTeR'});
runTest("textAlign spaces 1", {textAlign:' center'});
runTest("textAlign spaces 2", {textAlign:'center '});
runTest("textAlign spaces 3", {textAlign:' center '});
runTest("textAlign number", {textAlign:1});

runTest("textDecoration none", {textDecoration:'none'});
runTest("textDecoration underline", {textDecoration:'underline'});
runTest("textDecoration unknown", {textDecoration:'unknown'});
runTest("textDecoration underline case", {textDecoration:'Underline'});
runTest("textDecoration spaces 1", {textDecoration:' underline'});
runTest("textDecoration spaces 2", {textDecoration:'underline '});
runTest("textDecoration spaces 3", {textDecoration:' underline '});
runTest("textDecoration number", {textDecoration:1});

runTestNumerical("textIndent", function(n) { return {textIndent:n}; });
