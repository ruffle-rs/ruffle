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
 

START("13.5 XMLList Objects - XML only works with one item lists error");

var xl;
var result, expected, expectedStr;

xl = new XMLList("<a>1</a><a>2</a><a>3</a>");
expectedStr = "TypeError: Error #1086: The addNamespace method works only on lists containing one item.";
expected = "Error #1086";
result = "error, exception not thrown";

try{

xl.addNamespace('uri');
throw new Error("kXMLOnlyWorksWithOneItemLists error not thrown");

} catch( e1 ){

result = Utils.grabError(e1, e1.toString());

}

Assert.expectEq( "xmllist.addNamespace('uri')", expected, result );



xl = new XMLList("<a>1</a><a>2</a><a>3</a>");
expectedStr = "TypeError: Error #1086: The appendChild method works only on lists containing one item.";
expected = "Error #1086";
result = "error, exception not thrown";

try{

xl.appendChild("<e>f</e>");
throw new Error("kXMLOnlyWorksWithOneItemLists error not thrown");

} catch( e2 ){

result = Utils.grabError(e2, e2.toString());

}

Assert.expectEq( "xmllist.appendChild('<e>f</e>')", expected, result );



xl = new XMLList("<a>1</a><a>2</a><a>3</a>");
expectedStr = "TypeError: Error #1086: The inScopeNamespaces method works only on lists containing one item.";
expected = "Error #1086";
result = "error, exception not thrown";

try{

xl.inScopeNamespaces();
throw new Error("kXMLOnlyWorksWithOneItemLists error not thrown");

} catch( e3 ){

result = Utils.grabError(e3, e3.toString());

}

Assert.expectEq( "xmllist.inScopeNamespaces()", expected, result );



xl = new XMLList("<a>1</a><a>2</a><a>3</a>");
expectedStr = "TypeError: Error #1086: The insertChildAfter method works only on lists containing one item.";
expected = "Error #1086";
result = "error, exception not thrown";

try{

xl.insertChildAfter(null, "<s>t</s>");
throw new Error("kXMLOnlyWorksWithOneItemLists error not thrown");

} catch( e4 ){

result = Utils.grabError(e4, e4.toString());

}

Assert.expectEq( "xmllist.insertChildAfter(null, \"<s>t</s>\")", expected, result );



xl = new XMLList("<a>1</a><a>2</a><a>3</a>");
expectedStr = "TypeError: Error #1086: The insertChildBefore method works only on lists containing one item.";
expected = "Error #1086";
result = "error, exception not thrown";

try{

xl.insertChildBefore(null, "<s>t</s>");
throw new Error("kXMLOnlyWorksWithOneItemLists error not thrown");

} catch( e5 ){

result = Utils.grabError(e5, e5.toString());

}

Assert.expectEq( "xmllist.insertChildBefore(null, \"<s>t</s>\")", expected, result );



xl = new XMLList("<a>1</a><a>2</a><a>3</a>");
expectedStr = "TypeError: Error #1086: The name method works only on lists containing one item.";
expected = "Error #1086";
result = "error, exception not thrown";

try{

xl.name();
throw new Error("kXMLOnlyWorksWithOneItemLists error not thrown");

} catch( e6 ){

result = Utils.grabError(e6, e6.toString());

}

Assert.expectEq( "xmllist.name()", expected, result );



xl = new XMLList("<a>1</a><a>2</a><a>3</a>");
expectedStr = "TypeError: Error #1086: The getNamespace method works only on lists containing one item.";
expected = "Error #1086";
result = "error, exception not thrown";

try{

//xl.getNamespace("blah");
myGetNamespace(xl, "blah");
throw new Error("kXMLOnlyWorksWithOneItemLists error not thrown");

} catch( e7 ){

result = Utils.grabError(e7, e7.toString());

}

Assert.expectEq( "xmllist.getNamespace(\"blah\")", expected, result );



xl = new XMLList("<a>1</a><a>2</a><a>3</a>");
expectedStr = "TypeError: Error #1086: The namespaceDeclarations method works only on lists containing one item.";
expected = "Error #1086";
result = "error, exception not thrown";

try{

xl.namespaceDeclarations();
throw new Error("kXMLOnlyWorksWithOneItemLists error not thrown");

} catch( e8 ){

result = Utils.grabError(e8, e8.toString());

}

Assert.expectEq( "xmllist.namespaceDeclarations()", expected, result );



xl = new XMLList("<a>1</a><a>2</a><a>3</a>");
expectedStr = "TypeError: Error #1086: The nodeKind method works only on lists containing one item.";
expected = "Error #1086";
result = "error, exception not thrown";

try{

xl.nodeKind();
throw new Error("kXMLOnlyWorksWithOneItemLists error not thrown");

} catch( e9 ){

result = Utils.grabError(e9, e9.toString());

}

Assert.expectEq( "xmllist.nodeKind()", expected, result );



xl = new XMLList("<a>1</a><a>2</a><a>3</a>");
expectedStr = "TypeError: Error #1086: The removeNamespace method works only on lists containing one item.";
expected = "Error #1086";
result = "error, exception not thrown";

try{

xl.removeNamespace('pfx');
throw new Error("kXMLOnlyWorksWithOneItemLists error not thrown");

} catch( e10 ){

result = Utils.grabError(e10, e10.toString());

}

Assert.expectEq( "xmllist.removeNamespace('pfx')", expected, result );



xl = new XMLList("<a>1</a><a>2</a><a>3</a>");
expectedStr = "TypeError: Error #1086: The setChildren method works only on lists containing one item.";
expected = "Error #1086";
result = "error, exception not thrown";

try{

xl.setChildren("<c>4</c><c>5</c>");
throw new Error("kXMLOnlyWorksWithOneItemLists error not thrown");

} catch( e11 ){

result = Utils.grabError(e11, e11.toString());

}

Assert.expectEq( "xmllist.setChildren(\"<c>4</c><c>5</c>\")", expected, result );



xl = new XMLList("<a>1</a><a>2</a><a>3</a>");
expectedStr = "TypeError: Error #1086: The setLocalName method works only on lists containing one item.";
expected = "Error #1086";
result = "error, exception not thrown";

try{

xl.setLocalName("new local name");
throw new Error("kXMLOnlyWorksWithOneItemLists error not thrown");

} catch( e12 ){

result = Utils.grabError(e12, e12.toString());

}

Assert.expectEq( "xmllist.setLocalName(\"new local name\")", expected, result );



xl = new XMLList("<a>1</a><a>2</a><a>3</a>");
expectedStr = "TypeError: Error #1086: The setName method works only on lists containing one item.";
expected = "Error #1086";
result = "error, exception not thrown";

try{

xl.setName("myName");
throw new Error("kXMLOnlyWorksWithOneItemLists error not thrown");

} catch( e13 ){

result = Utils.grabError(e13, e13.toString());

}

Assert.expectEq( "xmllist.setName(\"myName\")", expected, result );



xl = new XMLList("<a>1</a><a>2</a><a>3</a>");
expectedStr = "TypeError: Error #1086: The setNamespace method works only on lists containing one item.";
expected = "Error #1086";
result = "error, exception not thrown";

try{

xl.setNamespace("pfx");
throw new Error("kXMLOnlyWorksWithOneItemLists error not thrown");

} catch( e14 ){

result = Utils.grabError(e14, e14.toString());

}

Assert.expectEq( "xmllist.setNamespace(\"pfx\")", expected, result );


END();

