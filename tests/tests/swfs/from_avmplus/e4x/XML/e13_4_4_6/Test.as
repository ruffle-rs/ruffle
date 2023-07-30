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

START("13.4.4.6 - XML child()");

//TEST(1, true, XML.prototype.hasOwnProperty("child"));

emps =
<employees>
    <employee id="0"><name>Jim</name><age>25</age></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
</employees>;

correct =
<employee id="0"><name>Jim</name><age>25</age></employee> +
<employee id="1"><name>Joe</name><age>20</age></employee>;


TEST(2, correct, emps.child("employee"));

TEST(3, <name>Joe</name>, emps.employee[1].child("name"));

correct = <employee id="1"><name>Joe</name><age>20</age></employee>;

TEST(4, correct, emps.child(1));

var xmlDoc = "<MLB><Team>Giants</Team><City>San Francisco</City><Team>Padres</Team><City>San Diego</City></MLB>";

// Rhino returns "<Team>Giants</Team>\n<Team>Padres></Team>"

// propertyName as a string
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.child('Team')", "<Team>Giants</Team>" + NL() + "<Team>Padres</Team>",
             (MYXML = new XML(xmlDoc), MYXML.child('Team').toString() ));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.child('Team') instanceof XMLList", true,
             (MYXML = new XML(xmlDoc), MYXML.child('Team') instanceof XMLList ));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.child('Team') instanceof XML", false,
             (MYXML = new XML(xmlDoc), MYXML.child('Team') instanceof XML ));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.child('Team').length()", 2,
             (MYXML = new XML(xmlDoc), MYXML.child('Team').length()));
Assert.expectEq( "MYXML = new XML(null), MYXML.child('Team')", "",
             (MYXML = new XML(null), MYXML.child('Team').toString() ));
Assert.expectEq( "MYXML = new XML(undefined), MYXML.child('Team')", "",
             (MYXML = new XML(undefined), MYXML.child('Team').toString() ));
Assert.expectEq( "MYXML = new XML(), MYXML.child('Team')", "",
             (MYXML = new XML(), MYXML.child('Team').toString() ));

// propertyName as a numeric index
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.child(1) instanceof XMLList", true,
             (MYXML = new XML(xmlDoc), MYXML.child(1) instanceof XMLList ));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.child(1)", "San Francisco",
             (MYXML = new XML(xmlDoc), MYXML.child(1).toString() ));
Assert.expectEq( "MYXML = new XML(null), MYXML.child(1)", "",
             (MYXML = new XML(null), MYXML.child(1).toString() ));
Assert.expectEq( "MYXML = new XML(undefined), MYXML.child(1)", "",
             (MYXML = new XML(undefined), MYXML.child(1).toString() ));
Assert.expectEq( "MYXML = new XML(), MYXML.child(1)", "",
             (MYXML = new XML(), MYXML.child(1).toString() ));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.child(0) instanceof XMLList", true,
             (MYXML = new XML(xmlDoc), MYXML.child(0) instanceof XMLList ));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.child(0)", "Giants",
             (MYXML = new XML(xmlDoc), MYXML.child(0).toString() ));
Assert.expectEq( "MYXML = new XML(null), MYXML.child(0)", "",
             (MYXML = new XML(null), MYXML.child(0).toString() ));
Assert.expectEq( "MYXML = new XML(undefined), MYXML.child(0)", "",
             (MYXML = new XML(undefined), MYXML.child(0).toString() ));
Assert.expectEq( "MYXML = new XML(), MYXML.child(0)", "",
             (MYXML = new XML(), MYXML.child(0).toString() ));

// propertyName is invalid

// invalid propertyName
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.child('DoesNotExist') instanceof XMLList", true,
             (MYXML = new XML(xmlDoc), MYXML.child('DoesNotExist') instanceof XMLList ));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.child('DoesNotExist')", "",
             (MYXML = new XML(xmlDoc), MYXML.child('DoesNotExist').toString() ));
Assert.expectEq( "MYXML = new XML(null), MYXML.child('DoesNotExist')", "",
             (MYXML = new XML(null), MYXML.child('DoesNotExist').toString() ));
Assert.expectEq( "MYXML = new XML(undefined), MYXML.child('DoesNotExist')", "",
             (MYXML = new XML(undefined), MYXML.child('DoesNotExist').toString() ));
Assert.expectEq( "MYXML = new XML(), MYXML.child('DoesNotExist')", "",
             (MYXML = new XML(), MYXML.child('DoesNotExist').toString() ));

// invalid index
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.child(8) instanceof XMLList", true,
             (MYXML = new XML(xmlDoc), MYXML.child(8) instanceof XMLList ));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.child(8)", "",
             (MYXML = new XML(xmlDoc), MYXML.child(8).toString() ));
Assert.expectEq( "MYXML = new XML(null), MYXML.child(8)", "",
             (MYXML = new XML(null), MYXML.child(8).toString() ));
Assert.expectEq( "MYXML = new XML(undefined), MYXML.child(8)", "",
             (MYXML = new XML(undefined), MYXML.child(8).toString() ));
Assert.expectEq( "MYXML = new XML(), MYXML.child(8)", "",
             (MYXML = new XML(), MYXML.child(8).toString() ));


END();
