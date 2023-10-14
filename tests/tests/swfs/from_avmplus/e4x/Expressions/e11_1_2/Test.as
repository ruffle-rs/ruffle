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

START("11.1.2 - Qualified Identifiers");

x1 =
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
    soap:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
    <soap:Body>
        <m:getLastTradePrice xmlns:m="http://mycompany.com/stocks">
            <symbol>DIS</symbol>
        </m:getLastTradePrice>
    </soap:Body>
</soap:Envelope>;

soap = new Namespace("http://schemas.xmlsoap.org/soap/envelope/");
stock = new Namespace("http://mycompany.com/stocks");

encodingStyle = x1.@soap::encodingStyle;
TEST_XML(1, "http://schemas.xmlsoap.org/soap/encoding/", encodingStyle);

correct =
<soap:Body xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
    <m:getLastTradePrice xmlns:m="http://mycompany.com/stocks">
        <symbol>DIS</symbol>
    </m:getLastTradePrice>
</soap:Body>;

body = x1.soap::Body;
TEST_XML(2, correct.toXMLString(), body);

body = x1.soap::["Body"];
TEST_XML(3, correct.toXMLString(), body);

q = new QName(soap, "Body");
body = x1[q];
TEST_XML(4, correct.toXMLString(), body);

correct =
<symbol xmlns:m="http://mycompany.com/stocks" xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">MYCO</symbol>;

x1.soap::Body.stock::getLastTradePrice.symbol = "MYCO";
TEST_XML(5, correct.toXMLString(), x1.soap::Body.stock::getLastTradePrice.symbol);

// SOAP messages
var msg1 = <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
        s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
    <s:Body>
        <m:GetLastTradePrice xmlns:m="http://mycompany.com/stocks/">
            <symbol>DIS</symbol>
        </m:GetLastTradePrice>
    </s:Body>
</s:Envelope>

var msg2 = <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
        s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
    <s:Body>
        <m:GetLastTradePrice xmlns:m="http://mycompany.com/stocks/">
            <symbol>MACR</symbol>
        </m:GetLastTradePrice>
    </s:Body>
</s:Envelope>

var msg3 = <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
        s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
    <s:Body>
        <m:GetLastTradePrice xmlns:m="http://mycompany.com/stocks/"
         m:blah="http://www.hooping.org">
            <symbol>MACR</symbol>
        </m:GetLastTradePrice>
    </s:Body>
</s:Envelope>

var msg4 = <soap>
    <bakery>
        <m:g xmlns:m="http://macromedia.com/software/central/"
        pea="soup"
        pill="box"
        neck="lace"
        m:blah="http://www.hooping.org">
            <pill>box</pill>
            <neck>lace</neck>
            <pea>soup</pea>
        </m:g>
    </bakery>
</soap>

var msg5 = "soupboxlacehttp://www.hooping.org";

// declare namespaces
var ns1 = new Namespace("http://schemas.xmlsoap.org/soap/envelope/");
var ns2= new Namespace ("http://mycompany.com/stocks/");
var ns3= new Namespace ("http://macromedia.com/software/central/");

// extract the soap encoding style and body from the soap msg1
var encodingStyle = msg1.@ns1::encodingStyle;
var stockURL = msg1.ns1::Body.ns2::GetLastTradePrice.@ns2::blah;

var body = msg1.ns1::Body;

// change the stock symbol
body.ns2::GetLastTradePrice.symbol = "MACR";


Assert.expectEq("body.ns2::GetLastTradePrice.symbol = \"MACR\"", "MACR",
           ( body.ns2::GetLastTradePrice.symbol.toString()));


bodyQ = msg1[QName(ns1, "Body")];

Assert.expectEq("ms1.ns1::Body == msg1[QName(ns1, \"Body\")]", true, (bodyQ == body));

Assert.expectEq("msg1 == msg2", true,
           ( msg1 == msg2));
           
Assert.expectEq("msg1.@ns1::encodingStyle", "http://schemas.xmlsoap.org/soap/encoding/",
           ( msg1.@ns1::encodingStyle.toString()));

Assert.expectEq("msg3.ns1::Body.ns2::GetLastTradePrice.@ns2", "http://www.hooping.org",
           ( msg3.ns1::Body.ns2::GetLastTradePrice.@ns2::blah.toString()));


// Rhino behaves differently:

Assert.expectEq("msg4.bakery.ns3::g.@*", msg5,
           ( msg4.bakery.ns3::g.@*.toString()));
           
var x1 = <x xmlns:ns="foo" ns:v='55'><ns:a>10</ns:a><b/><ns:c/></x>;
var ns = new Namespace("foo");

Assert.expectEq("x1.ns::*", new XMLList("<ns:a xmlns:ns=\"foo\">10</ns:a><ns:c xmlns:ns=\"foo\"/>").toString(), x1.ns::*.toString());

Assert.expectEq("x1.ns::a", "10", x1.ns::a.toString())

Assert.expectEq("x1.*::a", "10", x1.*::a.toString()); // issue: assert

Assert.expectEq("x1.ns::a", "20", (x1.ns::a = 20, x1.ns::a.toString()));

Assert.expectEq("x1.@ns::['v']", "55", x1.@ns::['v'].toString());

Assert.expectEq("x1.@ns::['v']", "555", (x1.@ns::['v'] = '555', x1.@ns::['v'].toString()));

var y1 = <y xmlns:ns="foo" a="10" b="20" ns:c="30" ns:d="40"/>
Assert.expectEq("y1.@ns::*.length()", 2, y1.@ns::*.length());

var z = new XMLList("<b xmlns:ns=\"foo\"/><ns:c xmlns:ns=\"foo\"/>");
Assert.expectEq("x1.*", z.toString(), (delete x1.ns::a, x1.*.toString()));

END();
