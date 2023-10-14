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

START("11.1.4 - XML Initializer");

XML.ignoreWhitespace = true;
person = <person><name>John</name><age>25</age></person>;
TEST(1, <person><name>John</name><age>25</age></person>, person);

e = <employees>
    <employee id = "1"><name>Joe</name><age>20</age></employee>
    <employee id = "2"><name>Sue</name><age>30</age></employee>
    </employees>;

TEST_XML(2, 1, e.employee[0].@id);

correct = <name>Sue</name>;
TEST(3, correct, e.employee[1].name);

names = new Array();
names[0] = "Alpha";
names[1] = "Bravo";
names[2] = "Charlie";
names[3] = "Delta";
names[4] = "Echo";
names[5] = "Golf";
names[6] = "Hotel";
names[7] = "India";
names[8] = "Juliet";
names[9] = "Kilo";

ages = new Array();
ages[0] = "20";
ages[1] = "21";
ages[2] = "22";
ages[3] = "23";
ages[4] = "24";
ages[5] = "25";
ages[6] = "26";
ages[7] = "27";
ages[8] = "28";
ages[9] = "29";

for (i = 0; i < 10; i++)
{
    e.*[i] = <employee id={i}>
           <name>{names[i].toUpperCase()}</name>
           <age>{ages[i]}</age>
           </employee>;

    correct = new XML("<employee id=\"" + i + "\"><name>" + names[i].toUpperCase() + "</name><age>" + ages[i] + "</age></employee>");
    TEST(4 + i, correct, e.*[i]);
}

tagName = "name";
attributeName = "id";
attributeValue = 5;
content = "Fred";

x1 = <{tagName} {attributeName}={attributeValue}>{content}</{tagName}>;
TEST(14, "<name id=\"5\">Fred</name>", x1.toXMLString());



// Test {} on XML and XMLList types
x1 =
<rectangle>
    <length>30</length>
    <width>50</width>
</rectangle>;

correct =
<rectangle>
    <width>50</width>
    <length>30</length>
</rectangle>;

x1 = <rectangle>{x1.width}{x1.length}</rectangle>;

TEST(15, correct, x1);

var content = "<foo name=\"value\">bar</foo>";

x1 = <x><a>{content}</a></x>;
correct = <x/>;
correct.a = content;
TEST(16, correct, x1);

x1 = <x a={content}/>;
correct = <x/>;
correct.@a = content;
TEST(17, correct, x1);

a = 5;
b = 3;
c = "x";
x1 = <{c} a={a + " < " + b + " is " + (a < b)}>{a + " < " + b + " is " + (a < b)}</{c}>;
TEST(18, "<x a=\"5 &lt; 3 is false\">5 &lt; 3 is false</x>", x1.toXMLString());

x1 = <{c} a={a + " > " + b + " is " + (a > b)}>{a + " > " + b + " is " + (a > b)}</{c}>;
TEST(19, "<x a=\"5 > 3 is true\">5 &gt; 3 is true</x>", x1.toXMLString());

var tagname = "name";
var attributename = "id";
var attributevalue = 5;
content = "Fred";
 
var xml1 = <{tagname} {attributename}={attributevalue}>{content}</{tagname}>;

Assert.expectEq( "x = <{tagname} {attributename}={attributevalue}>{content}</{tagname}>", true,
           (  x1 = new XML('<name id="5">Fred</name>'), (xml1 == x1)));
           


names = ["Alfred", "Allie", "Maryann", "Jason", "Amy", "Katja", "Antonio", "Melvin", "Stefan", "Amber"];
ages = [55, 21, 25, 23, 28, 30, 35, 26, 30, 30];

var xml2;
var xml2string = "<employees>";
var e = new Array();
for (i = 0; i < 10; i++) {
    e[i] = <employee id={i}>
        <name>{names[i].toUpperCase()}</name>
        <age>{ages[i]}</age>
    </employee>;
    xml2string += "<employee id=\"" + i + "\"><name>" + names[i].toUpperCase() + "</name><age>" + ages[i] + "</age></employee>";
}
xml2 = <employees>{e[0]}{e[1]}{e[2]}{e[3]}{e[4]}{e[5]}{e[6]}{e[7]}{e[8]}{e[9]}</employees>;
xml2string += "</employees>";

Assert.expectEq( "Evaluating expressions in a for loop", true,
           (  x1 = new XML(xml2string), (xml2 == x1)));
          

var xml3 = <person><name>John</name><age>25</age></person>;

Assert.expectEq( "x = <person><name>John</name><age>25</age></person>", true,
           (  x1 = new XML(xml3.toString()), (xml3 == x1)));

var xml4 = new XML("<xml><![CDATA[<hey>]]></xml>");

Assert.expectEq( "<xml><![CDATA[<hey>]]></xml>", true,
           (  x1 = new XML("<xml><![CDATA[<hey>]]></xml>"), (xml4 == x1)));
          
xml5 = <xml><b>heh            hey</b></xml>;
XML.ignoreWhitespace = true;

Assert.expectEq( "<xml><b>heh            hey</b></xml>", true,
           (  x1 = new XML("<xml><b>heh            hey</b></xml>"), (xml5 == x1)));


Assert.expectEq( "x = new XML(\"\"), xml = \"\", (xml == x)", true,
           (  x1 = new XML(""), xml = "", (xml == x1)));
           

       
var xx = new XML("<classRec><description><![CDATA[characteristics:<ul> <li>A</li> <li>B</li>   </ul>]]></description></classRec>");

Assert.expectEq( "<xml><![CDATA[characteristics:<ul> <li>A</li> <li>B</li>   </ul>]]></xml>, xml.toXMLString()",
             "<classRec>" + NL() + "  <description><![CDATA[characteristics:<ul> <li>A</li> <li>B</li>   </ul>]]></description>" + NL() + "</classRec>",
             xx.toXMLString());
             
Assert.expectEq( "<xml><![CDATA[characteristics:<ul> <li>A</li> <li>B</li>   </ul>]]></xml>, xml.description.text()",
             "characteristics:<ul> <li>A</li> <li>B</li>   </ul>",
             xx.description.text().toString());

Assert.expectEq( "<xml><![CDATA[characteristics:<ul> <li>A</li> <li>B</li>   </ul>]]></xml>, xml.description.child(0)",
             "characteristics:<ul> <li>A</li> <li>B</li>   </ul>",
             xx.description.child(0).toString());
             
Assert.expectEq( "<xml><![CDATA[characteristics:<ul> <li>A</li> <li>B</li>   </ul>]]></xml>, xml.description.child(0).nodeKind()",
             "text",
             xx.description.child(0).nodeKind());
             
var desc = "this is the <i>text</i>";

x1 = <description>{"<![CDATA[" + desc + "]]>"}</description>;

Assert.expectEq("desc = \"this is the <i>text</i>\"; x = <description>{\"<![CDATA[\" + desc + \"]]>\"}</description>;",
            "<![CDATA[this is the <i>text</i>]]>", x1.toString());


Assert.expectEq("desc = \"this is the <i>text</i>\"; x = <description>{\"<![CDATA[\" + desc + \"]]>\"}</description>;",
            "<description>&lt;![CDATA[this is the &lt;i&gt;text&lt;/i&gt;]]&gt;</description>", x1.toXMLString());
            
// Testing for extra directives. See bug 94230.
var xx = <?xml version="1.0" encoding="UTF-8"?>
<?mso-infoPathSolution solutionVersion="1.0.0.26" productVersion="11.0.6250" PIVersion="1.0.0.0" href="file:///C:\Documents%20and%20Settings\Bob\BoB\Goodbye%20Doubt\Repository\CMS\Forms\PatternForm.xsn" name="urn:schemas-microsoft-com:office:infopath:PatternForm:urn-axiology-PatternDocument" language="en-us" ?>
<?mso-application progid="InfoPath.Document"?>;

Assert.expectEq("Testing for extra directives", "", xx.toString());

xx = new XML("<?xml version=\"1.0\" encoding=\"UTF-8\"?><?mso-infoPathSolution solutionVersion=\"1.0.0.26\" productVersion=\"11.0.6250\" PIVersion=\"1.0.0.0\" href=\"file:///C:\Documents%20and%20Settings\Bob\BoB\Goodbye%20Doubt\Repository\CMS\Forms\PatternForm.xsn\" name=\"urn:schemas-microsoft-com:office:infopath:PatternForm:urn-axiology-PatternDocument\" language=\"en-us\" ?><?mso-application progid=\"InfoPath.Document\"?>");

Assert.expectEq("Testing for extra directives", "", xx.toString());

END();
