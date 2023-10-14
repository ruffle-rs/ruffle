/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 4 -*-
 *
 * ***** BEGIN LICENSE BLOCK *****
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
// var SECTION = "tostring_001"

/* TO-DO: commenting printStatus and printBugNumber
printStatus ("Number formatting test.");
printBugNumber ("11178");
*/
var testcases = getTestCases();

function getTestCases()
{
    var array = new Array();
    var item = 0;
    
    var n0 = 1e23;
    var n1 = 5e22;
    var n2 = 1.6e24;

    array[item++] = Assert.expectEq( "1e+23", n0.toString(),"1e+23" );
    
    array[item++] = Assert.expectEq( "5e+22", n1.toString(), "5e+22");
    
    array[item++] = Assert.expectEq( "1.6e+24", n2.toString(), "1.6e+24");
    
    return array;
}


