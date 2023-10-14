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

/*
*
*
* See http://bugzilla.mozilla.org/show_bug.cgi?id=616125
*
*/
//-----------------------------------------------------------------------------

// var SECTION = "616125";
// var VERSION = "";
// var TITLE   = "Need coverage around XML.setNotification(function)";
// var bug = "616125";



var myXML:XML = <foo/>;

myXML.setNotification(notifier1);
myXML.@attrib1 = "foz"; // attributeAdded
function notifier1(targetCurrent:Object, command:String, target:Object, value:Object, detail:Object):void {
    Assert.expectEq("attributeAdded-command", "attributeAdded", command);
    Assert.expectEq("attributeAdded-value", "attrib1", value);
    Assert.expectEq("attributeAdded-detail", "foz", detail);
}

myXML.setNotification(notifier2);
myXML.@attrib1 = "baz"; // attributeChanged
function notifier2(targetCurrent:Object, command:String, target:Object, value:Object, detail:Object):void {
    Assert.expectEq("attributeChanged-command", "attributeChanged", command);
    Assert.expectEq("attributeChanged-value", "attrib1", value);
    Assert.expectEq("attributeChanged-detail", "foz", detail);
}

myXML.setNotification(notifier3);
delete myXML.@attrib1;  // attributeRemoved
function notifier3(targetCurrent:Object, command:String, target:Object, value:Object, detail:Object):void {
    Assert.expectEq("attributeRemoved-command", "attributeRemoved", command);
    Assert.expectEq("attributeRemoved-value", "attrib1", value);
    Assert.expectEq("attributeRemoved-detail", "baz", detail);
}

myXML.setNotification(notifier4);
myXML.appendChild(<node1/>); // nodeAdded
function notifier4(targetCurrent:Object, command:String, target:Object, value:Object, detail:Object):void {
    Assert.expectEq("nodeAdded-command", "nodeAdded", command);
}

myXML.setNotification(notifier5);
myXML.node1 = <node2/>;      // nodeChanged
function notifier5(targetCurrent:Object, command:String, target:Object, value:Object, detail:Object):void {
    Assert.expectEq("nodeChanged-command", "nodeChanged", command);
}

myXML.setNotification(notifier6);
delete myXML.node2[0];       // nodeRemoved
function notifier6(targetCurrent:Object, command:String, target:Object, value:Object, detail:Object):void {
    Assert.expectEq("nodeRemoved-command", "nodeRemoved", command);
}

myXML.setNotification(notifier7);
myXML.appendChild(<node3/>);     // nodeAdded
function notifier7(targetCurrent:Object, command:String, target:Object, value:Object, detail:Object):void {
    Assert.expectEq("nodeAdded-command", "nodeAdded", command);
}

myXML.setNotification(notifier8);
myXML.node3 = "some text";       // textSet
function notifier8(targetCurrent:Object, command:String, target:Object, value:Object, detail:Object):void {
    Assert.expectEq("textSet-command", "textSet", command);
}

myXML.setNotification(notifier9);
myXML.node3[0].setName("node4"); //nameSet
function notifier9(targetCurrent:Object, command:String, target:Object, value:Object, detail:Object):void {
    Assert.expectEq("nameSet-command", "nameSet", command);
}

myXML.setNotification(notifier10);
myXML.setNamespace(new Namespace("fozbaz")); // namespaceSet
function notifier10(targetCurrent:Object, command:String, target:Object, value:Object, detail:Object):void {
    Assert.expectEq("namespaceSet-command", "namespaceSet", command);
}

// Notifiers MUST be functions or null
var err:String = "no error";
try {
    var strz:String = "string"
    myXML.setNotification(strz);

} catch (e:Error) {
    err = Utils.grabError(e, e.toString());
} finally {
    Assert.expectEq("StringNotifier", "Error #1034", err );
}


