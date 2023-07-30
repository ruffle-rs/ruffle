/* -*- Mode: java; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 4 -*-
 *
 * ***** BEGIN LICENSE BLOCK *****
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
public class Test {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

function START(summary)
{
      // print out bugnumber

     /*if ( BUGNUMBER ) {
              writeLineToLog ("BUGNUMBER: " + BUGNUMBER );
      }*/
    XML.setSettings (null);
    testcases = new Array();

    // text field for results
    tc = 0;
    /*this.addChild ( tf );
    tf.x = 30;
    tf.y = 50;
    tf.width = 200;
    tf.height = 400;*/

    //_print(summary);
    var summaryParts = summary.split(" ");
    //_print("section: " + summaryParts[0] + "!");
    //fileName = summaryParts[0];

}

function TEST(section, expected, actual)
{
    AddTestCase(section, expected, actual);
}
 

function TEST_XML(section, expected, actual)
{
  var actual_t = typeof actual;
  var expected_t = typeof expected;

  if (actual_t != "xml") {
    // force error on type mismatch
    TEST(section, new XML(), actual);
    return;
  }

  if (expected_t == "string") {

    TEST(section, expected, actual.toXMLString());
  } else if (expected_t == "number") {

    TEST(section, String(expected), actual.toXMLString());
  } else {
    reportFailure ("", 'Bad TEST_XML usage: type of expected is "+expected_t+", should be number or string');
  }
}

function reportFailure (section, msg)
{
  trace("~FAILURE: " + section + " | " + msg);
}

function AddTestCase( description, expect, actual ) {
   testcases[tc++] = Assert.expectEq(description, "|"+expect+"|", "|"+actual+"|" );
}

function myGetNamespace (obj, ns) {
    if (ns != undefined) {
        return obj.namespace(ns);
    } else {
        return obj.namespace();
    }
}




function NL()
{
  //return java.lang.System.getProperty("line.separator");
  return "\n";
}


function BUG(arg){
  // nothing here
}

function END()
{
    //test();
}


function grabError(arg1, arg2){
  return arg2;
}
 

START("13.4 XML Object - Miscellaneous errors");

function grabError(err, str) {
    var typeIndex = str.indexOf("Error:");
    var type = str.substr(0, typeIndex + 5);
    if (type == "TypeError") {
        Assert.expectEq("Asserting for TypeError", true, (err is TypeError));
    }
    var numIndex = str.indexOf("Error #");
    var num = str.substr(numIndex, 11);
    return num;
}

var expected, result;

// missing quote after third

expectedStr = "TypeError: Error #1095: XML parser failure: Unterminated attribute";
expected = "Error #1095";

try{

var x1 = new XML("<song which=\"third><cd>Blue Train</cd><title>Locomotion</title><artist>John Coltrane</artist></song>");
throw "missing \"";

} catch( e1 ){

result = grabError(e1, e1.toString());

}
 
Assert.expectEq( "missing \" after attribute", expected, result );


// missing quotes around third

expectedStr = "TypeError: Error #1090: XML parser failure: element is malformed";
expected = "Error #1090";

try{

x1 = new XML("<song which=third><cd>Blue Train</cd><title>Locomotion</title><artist>John Coltrane</artist></song>");
throw "missing quotes around attribute";

} catch( e2 ){

result = grabError(e2, e2.toString());

}
 
Assert.expectEq( "missing quotes around attribute", expected, result );


// missing starting quote around third

expectedStr = "TypeError: Error #1090: XML parser failure: element is malformed";
expected = "Error #1090";

try{

x1 = new XML("<song which=third\"><cd>Blue Train</cd><title>Locomotion</title><artist>John Coltrane</artist></song>");
throw "missing starting quote for attribute";

} catch( e3 ){

result = grabError(e3, e3.toString());

}
 
Assert.expectEq( "missing starting quote for attribute", expected, result );



// missing ! at start of CDATA section

expectedStr = "TypeError: Error #1090: XML parser failure: element is malformed";
expected = "Error #1090";

try{

x1 = new XML("<songlist> <[CDATA[    <?xml version='1.0'?>   <entry>           <name>John Doe</name>         <email href='mailto:jdoe@somewhere.com'/> </entry>]]>       <![CDATA[   count = 0; function incCount() {   count++;    }]]></songlist>");
throw "missing ! at start of CDATA section";

} catch( e4 ){

result = grabError(e4, e4.toString());

}
 
Assert.expectEq( "missing ! at start of CDATA section", expected, result );


// unterminated CDATA

expectedStr = "TypeError: Error #1091: XML parser failure: Unterminated CDATA section";
expected = "Error #1091";

try{

x1 = new XML("<songlist> <![CDATA[   <?xml version='1.0'?>   <entry>           <name>John Doe</name>         <email href='mailto:jdoe@somewhere.com'/> </entry>]>       <![CDATA[   count = 0; function incCount() {   count++;    }]></songlist>");
throw "unterminated CDATA section";

} catch( e5 ){

result = grabError(e5, e5.toString());

}
 
Assert.expectEq( "unterminated CDATA section", expected, result );


// unterminated comment

expectedStr = "TypeError: Error #1094: XML parser failure: Unterminated comment";
expected = "Error #1094";

try{

x1 = new XML("<!-- Alex Homer's Book List with Linked Schema");
throw "unterminated comment";

} catch( e6 ){

result = grabError(e6, e6.toString());

}
 
Assert.expectEq( "unterminated comment", expected, result );


// unterminated doctype

expectedStr = "TypeError: Error #1093: XML parser failure: Unterminated DOCTYPE declaration";
expected = "Error #1093";

try{

x1 = new XML("<!DOCTYPE PUBLIST [");
throw "unterminated doctype";

} catch( e7 ){

result = grabError(e7, e7.toString());

}
 
Assert.expectEq( "unterminated doctype", expected, result );


// bad attribute (E4X returns malformed element)

expectedStr = "TypeError: Error #1090: XML parser failure: element is malformed";
expected = "Error #1090";

try{

x1 = new XML("<song which='third'><cd>Blue Train</cd>  <title Locomotion</title> <artist>John Coltrane</artist></song>");
throw "bad attribute";

} catch( e8 ){

result = grabError(e8, e8.toString());

}
 
Assert.expectEq( "bad attribute", expected, result );


// "cd" must be terminated by "/cd"

expectedStr = "TypeError: Error #1085: The element type \"cd\" must be terminated by the matching end-tag \"</cd>\".";
expected = "Error #1085";

try{

x1 = new XML("<song which='third'><cd>Blue Train<title>Locomotion</title> <artist>John Coltrane</artist></song>");
throw "unterminated tag";

} catch( e9 ){

result = grabError(e9, e9.toString());

}
 
Assert.expectEq( "unterminated tag", expected, result );


// "song" must be terminated by "/song"

expectedStr = "TypeError: Error #1085: The element type \"song\" must be terminated by the matching end-tag \"</song>\".";
expected = "Error #1085";

try{

x1 = new XML("<song which='third'><cd>Blue Train</cd></title><artist>John Coltrane</artist></song>");
throw "mismatched end tag";

} catch( e10 ){

result = grabError(e10, e10.toString());

}
 
Assert.expectEq( "mismatched end tag", expected, result );


// "song" must be terminated by "/song"

expectedStr = "TypeError: Error #1085: The element type \"song\" must be terminated by the matching end-tag \"</song>\".";
expected = "Error #1085";

try{

x1 = new XML("<song which='third'><cd>Blue Train</cd></title><artist>John Coltrane</artist></SONG>");
throw "wrong case in root end tag";

} catch( e11 ){

result = grabError(e11, e11.toString());

}
 
Assert.expectEq( "wrong case in root end tag", expected, result );


// "cd" must be terminated by "/cd"

expectedStr = "TypeError: Error #1085: The element type \"cd\" must be terminated by the matching end-tag \"</cd>\".";
expected = "Error #1085";

try{

x1 = new XML("<song which='third'><cd>Blue Train</CD></title><artist>John Coltrane</artist></song>");
throw "wrong case in end tag";

} catch( e12 ){

result = grabError(e12, e12.toString());

}
 
Assert.expectEq( "wrong case end tag", expected, result );


// Rhino: Attribute name "name" associated with an element type "cd" must be followed by the "=" character

// E4X: element is malformed

expectedStr = "TypeError: Error #1090: XML parser failure: element is malformed";
expected = "Error #1090";

try{

x1 = new XML("<song which='third'><cd name>Blue Train</cd></title><artist>John Coltrane</artist></SONG>");
throw "missing attribute value";

} catch( e13 ){

result = grabError(e13, e13.toString());

}
 
Assert.expectEq( "missing attribute value", expected, result );



// E4X: unterminated XML decl

expectedStr = "TypeError: Error #1092: XML parser failure: Unterminated XML declaration";
expected = "Error #1092";

try{

x1 = new XML("<?xml version='1.0' encoding='UTF-8' standalone='no'><foo></foo>");
throw "unterminated XML decl";

} catch( e14 ){

result = grabError(e14, e14.toString());

}
 
Assert.expectEq( "unterminated XML decl", expected, result );


// Rhino: XML document structures must start and end within the same entity:

// E4X: same error

expectedStr = "TypeError: Error #1088: The markup in the document following the root element must be well-formed.";
expected = "Error #1088";

try{

x1 = new XML("<a>foo</a><b>bar</b>");
throw "XML must start and end with same entity";

} catch( e15 ){

result = grabError(e15, e15.toString());

}
 
Assert.expectEq( "XML must start and end with same entity", expected, result );



// Rhino: XML document structures must start and end within the same entity:

// E4X: same error

var y1 = new XMLList ("<a>foo</a><b>bar</b>");

expectedStr = "TypeError: Error #1088: The markup in the document following the root element must be well-formed.";
expected = "Error #1088";

try{

x1 = new XML(y1);
throw "XML must start and end with same entity";

} catch( e16 ){

result = grabError(e16, e16.toString());

}
 
Assert.expectEq( "XML must start and end with same entity", expected, result );


END();
