/* -*- mode: java; tab-width: 4 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//-----------------------------------------------------------------------------

// var SECTION = "E4XNode__deepCopy";
// var VERSION = "";
// var TITLE   = "Shouldn't crash on high XML trees upon invoking E4XNode::_deepCopy and E4X::_equals";

var testcases = getTestCases();

function createXML(depth)
{
    // char code for the first letter of the abc
    var FIRSTCHARCODE = 97;
    // lenght of the abc
    var ABCLENGTH = 26;

    // string holding the open tags
    var openTagList = "";
    var closeTagList = new Vector.<String>();

    for (var i = 0; i < depth; ++i) {
    var tagContent = "";
    var tagNameLength = i / ABCLENGTH;

    for (var j = 0; j <= tagNameLength; ++j) {
        tagContent += String.fromCharCode(FIRSTCHARCODE + i % ABCLENGTH);
    }

    openTagList += "<" + tagContent + ">";
    closeTagList.push("</" + tagContent + ">");
    }

    var xmlstring = openTagList + "sample";

    for each (var closeTag in closeTagList.reverse()) {
    xmlstring += closeTag;
    }

    return new XML(xmlstring);
}

/*
 * The test creates a big XML tree, then calls copy() to trigger the execution of E4XNode::_deepCopy.
 * Upon checking the result against the expected value, _equals is also called.
 * */

function getTestCases()
{
    var array = new Array();
    var item = 0;

    var xml = createXML(1000);

    try {
    var copied = xml.copy();
    var res = xml == copied;
    array[item++] = Assert.expectEq( "new XML(...).copy()", true, res);
    }
    catch (e: Error) {
    if (e.message.match("#1023"))
        array[item++] = Assert.expectEq( "new XML(...).copy()", 0, 0);
    else
        throw(e);
    }

    try {
    //dummy test, execution of toXMLString is the goal here
    res = xml.toXMLString().indexOf("sample") > 0;

    array[item++] = Assert.expectEq( "new XML(...).toXMLString()", true, res);
    }
    catch (e: Error) {
    if (e.message.match("#1023"))
        array[item++] = Assert.expectEq( "new XML(...).toXMLString()", 0, 0);
    else
        throw(e);
    }

    try {
    //dummy test, execution of descendants is the goal here
    res = xml.descendants().length() > 0;

    array[item++] = Assert.expectEq( "new XML(...).descendants()", true, res);
    }
    catch (e: Error) {
    if (e.message.match("#1023"))
        array[item++] = Assert.expectEq( "new XML(...).descendants()", 0, 0);
    else
        throw(e);
    }

    return array;
}
