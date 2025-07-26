/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

/*
*
* See http://bugzilla.mozilla.org/show_bug.cgi?id=598683
*
*/
//-----------------------------------------------------------------------------

import com.adobe.test.Utils;
import com.adobe.test.Assert;
// var SECTION = "598683";
// var VERSION = "";
// var TITLE   = "Bad XML with unterminated node with namespace not throwing correctly";
// var bug = "598683";

var testcases = getTestCases();

function getTestCases() {

    var actual:String;
    try
    {
        var xml:XML=new XML("<a><b:c xmlns:b=\"abc\"></d:c></a>") ;
        actual = xml.toXMLString();
    }
    catch(e)
    {
        actual = Utils.grabError(e, e.toString());
    }

    var expect:String= "Error #1085"; // kXMLUnterminatedElementTag

    var status:String = "new XML(\"<a><b:c xmlns:b=\"abc\"></d:c></a>\")";
    var array = new Array();
    array[0] = Assert.expectEq( status, expect, actual);

    return array;
}
