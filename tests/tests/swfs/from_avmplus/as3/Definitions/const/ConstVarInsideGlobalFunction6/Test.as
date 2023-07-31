/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Definitions\const";                  // provide a document reference (ie, ECMA section)
// var VERSION = "ActionScript 3.0";           // Version of JavaScript or ECMA
// var TITLE   = "Initialize a local const in function which contains an activation";       // Provide ECMA section title or a description
var BUGNUMBER = "";


function constTypedFormal(arg0 : uint) : uint
{
    const c0 : uint = g(arg0);

    function g(x:uint) : uint
    {
        return x;
    }

    return c0;
}

function constUnTypedFormal(arg0)
{
    const c0 = g(arg0);

    function g(x)
    {
        return x;
    }

    return c0;
}

function constTypedConst()
{
    const c0 : uint = 13;

    // keep this function here so an activation is generated
    function g(x:uint) : uint
    {
        return x;
    }

    return c0;
}

function constUnTypedConst()
{
    const c0 = 14;

    // keep this function here so an activation is generated
    function g(x)
    {
        return x;
    }

    return c0;
}

Assert.expectEq("Initialize global typed function local const with formal", 11, constTypedFormal(11));
Assert.expectEq("Initialize global untyped function local const with formal", 12, constUnTypedFormal(12));
Assert.expectEq("Initialize global typed function local const with a const", 13, constTypedConst());
Assert.expectEq("Initialize global untyped function local const with a const", 14, constUnTypedConst());

