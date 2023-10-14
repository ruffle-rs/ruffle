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

// var SECTION = "apply regression";
// var VERSION = "ECMA_3";

trace("STATUS: f.apply crash test.");

trace("BUGNUMBER: 21836");

var testcases = getTestCases();


function f()
{
}

function getTestCases() {
    var array = new Array();
    var item = 0;
    var thisError="no error";
    trace("The second argument of f.apply() should be an array");

    function doTest():String
    {
        //f.apply(2,2);
    
        try{
            f.apply(2,2);
        }catch(e:Error){
            thisError=(e.toString()).substring(0,22);
        }finally{
    
        }
        return thisError;
    }

    doTest();

    array[item++] = Assert.expectEq( "Make sure we don't crash", true, true);

    return array;
}
