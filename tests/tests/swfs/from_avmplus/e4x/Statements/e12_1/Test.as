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

START("12.1 - Default XML Namespace");


// Declare some namespaces ad a default namespace for the current block
var soap = new Namespace("soap", "http://schemas.xmlsoap.org/soap/envelope/");
var stock = new Namespace("stock", "http://mycompany.com/stocks");
default xml namespace = "http://schemas.xmlsoap.org/soap/envelope/";

// Create an XML initializer in the default (i.e., soap) namespace
message =
<Envelope 
    xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
    soap:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
    <Body>
        <stock:GetLastTradePrice xmlns:stock="http://mycompany.com/stocks">
            <stock:symbol>DIS</stock:symbol>
        </stock:GetLastTradePrice>
    </Body>
</Envelope>;

// Extract the soap encoding style using a QualifiedIdentifier
encodingStyle = message.@soap::encodingStyle;
TEST_XML(1, "http://schemas.xmlsoap.org/soap/encoding/", encodingStyle);

// Extract the body from the soap message using the default namespace
correct = 
<soap:Body
    xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
    <stock:GetLastTradePrice xmlns:stock="http://mycompany.com/stocks">
        <stock:symbol>DIS</stock:symbol>
    </stock:GetLastTradePrice>
</soap:Body>;

body = message.soap::Body;
TEST_XML(2, correct.toXMLString(), body);

// Change the stock symbol using the default namespace and qualified names
correct =
<soap:Envelope 
    xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
    soap:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
    <soap:Body>
        <stock:GetLastTradePrice xmlns:stock="http://mycompany.com/stocks">
            <stock:symbol>MYCO</stock:symbol>
        </stock:GetLastTradePrice>
    </soap:Body>
</soap:Envelope>;

message.soap::Body.stock::GetLastTradePrice.stock::symbol = "MYCO";

TEST(3, correct, message);

function scopeTest()
{
    var x1 = <a/>;
    //TEST(4, soap.uri, x1.getNamespace().uri);
    // Jeff says this is how it's supposed to work. Rhino is different.
    /*
    So what is going on here is that we�ve implemented a 
    proposed behavior, and Rhino implements the currently spec�d behavior. 
    The currently spec�d behavior is that the default default-xml-namespace 
    is the default-xml-namespace of global scope at the time of the call 
    until it is set by a local DXNS statement. This is hard to compile 
    and hard to read, so the working group has agreed to change 
    this behavior in two ways: 1/determine the inner default DXNS lexically 
    (by looking at what DXNS statement that comes most immediately before the 
    current function definition); and 2/if the local scope has a DXNS, the 
    default DXNS is the unnamed namespace (uri="").
    
    Jd
    */
    TEST(4, "", myGetNamespace(x1).uri);
    someuri = new Namespace("someuri", "http://someuri.org");	
    default xml namespace = "http://" + "someuri.org";
    x1 = <a/>;
    //TEST(5, "http://someuri.org", x1.getNamespace().uri);
    TEST(5, "http://someuri.org", myGetNamespace(x1).uri);
}

scopeTest();
default xml namespace = soap;
x1 = <a><b><c xmlns="">foo</c></b></a>;
//TEST(6, soap.uri, x.getNamespace().uri);
TEST(6, soap.uri, myGetNamespace(x1).uri);
//TEST(7, soap.uri, x.b.getNamespace().uri);
TEST(7, soap.uri, myGetNamespace(x1.b).uri);

ns = new Namespace("");
TEST(8, "foo", x1.b.ns::c.toString());

x1 = <a foo="bar"/>;
//TEST(9, soap.uri, x1.getNamespace().uri);
TEST(9, soap.uri, myGetNamespace(x1).uri);
//TEST(10, "", x.@foo.getNamespace().uri);
TEST(10, "", myGetNamespace(x1.@foo).uri);
TEST_XML(11, "bar", x1.@foo);

default xml namespace = "";
x1 = <x/>;
ns = new Namespace("sui", "http://someuri");
default xml namespace = ns;
x1.a = "foo";
TEST(12, "foo", x1["a"].toString());
q = new QName("a");
TEST(13, "foo", x1[q].toString());

default xml namespace = "";
x1[q] = "bar";
TEST(14, "bar", x1.ns::a.toString());

XML.prettyPrinting = false;
function f() {
	var ns = new Namespace("bar");
	default xml namespace = ns;
	x1 = <x><a/><b/><c/></x>;
	Assert.expectEq("Namespaces in function scope: ", 3, x1.ns::*.length());
}
var ns = new Namespace("foo");
default xml namespace = ns;
var x1 = <x><a/><b/><c/></x>;

Assert.expectEq("Namespaces in global scope: ", 3, x1.ns::*.length());
f();

XML.prettyPrinting = true;

/*
var x1  = new XML("<a xmlns:XMLNameSpace='http://www.macromedia.com' />");
var x_ = x.GetDefaultNameSpace();
var y1  = "XMLNameSpace";

Assert.expectEq( "default xml namespace :", true, (x_==y1) );
*/

END();
