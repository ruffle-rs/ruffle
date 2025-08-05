/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
*
* Date:    12 Feb 2002
* SUMMARY: Don't crash on invalid regexp literals /  \\/  /
*
* See http://bugzilla.mozilla.org/show_bug.cgi?id=122076
* The function checkURL() below sometimes caused a compile-time error:
*
*         SyntaxError: unterminated parenthetical (:
*
* However, sometimes it would cause a crash instead. The presence of
* other functions below is merely fodder to help provoke the crash.
* The constant |STRESS| is number of times we'll try to crash on this.
*
*/
//-----------------------------------------------------------------------------
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}


// var SECTION = "eregress_122076";
// var VERSION = "";
// var TITLE   = "Don't crash on invalid regexp literals /  \\/  /";
// var bug = "122076";

import com.adobe.test.Assert;
var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var STRESS = 10;

    var thisError = "no exceptions";
    var testRegExp = new TestRegExp("test_string");

    for (var i=0; i<STRESS; i++)
    {
        try
        {
            testRegExp.checkDate();
            testRegExp.checkDNSName();
            testRegExp.checkEmail();
            testRegExp.checkHostOrIP();
            testRegExp.checkIPAddress();
            testRegExp.checkURL();
        }
        catch(e)
        {
            thisError = "exception occurred";
        }
    }

    array[item++] = Assert.expectEq( "Test completion status", "no exceptions", thisError);

    return array;
}

class TestRegExp {
    var value;

    function TestRegExp(v)
    {
        this.value = v;
    }

    function checkDate()
    {
      return (this.value.search("/^[012]?\d\/[0123]?\d\/[0]\d$/") != -1);
    }

    function checkDNSName()
    {
      return (this.value.search("/^([\w\-]+\.)+([\w\-]{2,3})$/") != -1);
    }

    function checkEmail()
    {
      return (this.value.search("/^([\w\-]+\.)*[\w\-]+@([\w\-]+\.)+([\w\-]{2,3})$/") != -1);
    }

    function checkHostOrIP()
    {
      if (this.value.search("/^([\w\-]+\.)+([\w\-]{2,3})$/") == -1)
        return (this.value.search("/^[1-2]?\d{1,2}\.[1-2]?\d{1,2}\.[1-2]?\d{1,2}\.[1-2]?\d{1,2}$/") != -1);
      else
        return true;
    }

    function checkIPAddress()
    {
      return (this.value.search("/^[1-2]?\d{1,2}\.[1-2]?\d{1,2}\.[1-2]?\d{1,2}\.[1-2]?\d{1,2}$/") != -1);
    }

    function checkURL()
    {
      return (this.value.search("/^(((https?)|(ftp)):\/\/([\-\w]+\.)+\w{2,4}(\/[%\-\w]+(\.\w{2,})?)*(([\w\-\.\?\\/\*\$+@&#;`~=%!]*)(\.\w{2,})?)*\/?)$/") != -1);
    }
}
