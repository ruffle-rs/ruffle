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

START("11.1.3 - Wildcard Identifiers");

var x1 =
<alpha>
    <bravo>one</bravo>
    <charlie>two</charlie>
</alpha>

var correct = new XMLList("<bravo>one</bravo><charlie>two</charlie>");
TEST(1, correct, x1.*);

XML.ignoreWhitespace = false;

var xml1 = "<p><a>1</a><a>2</a><a>3</a></p>";
var xml2 = "<a>1</a><a>2</a><a>3</a>";
var xml3 = "<a><e f='a'>hey</e><e f='b'> </e><e f='c'>there</e></a><a>2</a><a>3</a>";
var xml4 = "<p><a hi='a' he='v' ho='m'>hello</a></p>";

var ns1 = new Namespace('foobar', 'http://boo.org');
var ns2 = new Namespace('fooboo', 'http://foo.org');
var ns3 = new Namespace('booboo', 'http://goo.org');

Assert.expectEq("x.a.*", "123", (x1 = new XML(xml1), x1.a.*.toString()));

Assert.expectEq("xmllist.a.*", "123", (x1 = new XMLList(xml1), x1.a.*.toString()));

Assert.expectEq("xmllist.*", "123", (x1 = new XMLList(xml2), x1.*.toString()));

Assert.expectEq("xmllist[0].*", "1", (x1 = new XMLList(xml2), x1[0].*.toString()));

Assert.expectEq("xmllist[0].e.*", "hey there", (x1 = new XMLList(xml3), x1[0].e.*.toString()));

Assert.expectEq("xml.a.@*", "avm", (x1 = new XML(xml4), x1.a.@*.toString()));


END();
