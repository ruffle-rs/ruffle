/* -*- Mode: javascript; c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "apply_001";
// var VERSION = "";

// var TITLE = "Function.prototype.apply with very long argument lists";


var testcases = getTestCases();

function getTestCases()
{
    var array = new Array();
    var initial = 1000;
    var limit = 10000000;
    var multiplier = 10;

    function makeArray(n) {
        var A = new Array;
        for ( var i=0 ; i < n ; i++ )
            A[i] = i+1;
        return A;
    }

    function sum(...rest) {
        var n = 0;
        for ( var i=0 ; i < rest.length ; i++ )
            n += rest[i];
        return n;
    }

    for ( var i=initial ; i < limit ; i *= multiplier ) {
        status = 'Test apply(bigarray) #' + i;
        actual = sum.apply(null, makeArray(i));
        expect = (i*(i+1))/2;
        array.push(Assert.expectEq( status, expect, actual));
    }

    return array;
}
