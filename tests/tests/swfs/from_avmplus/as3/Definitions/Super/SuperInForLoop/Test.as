/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

// Regression test for CMP-1695.

class Zeeb
{
    public var moo;
}

class Zonk extends Zeeb
{
    public function for_array_access(zam)
    {
        var result = []
        for ( super['moo'] in zam ) {
            result.push(super.moo);
        }
        Assert.expectEq('for_array_access:length', true, result.indexOf('length') != -1 );
        Assert.expectEq('for_array_access:company', true, result.indexOf('company') != -1 );
        Assert.expectEq('for_array_access:year', true, result.indexOf('year') != -1 );
        Assert.expectEq('for_array_access:0', true, result.indexOf(0) != -1 );
    }

    public function foreach_array_access(zam)
    {
        var result = []
        for each ( super['moo'] in zam ) {
            result.push(super.moo);
        }
        Assert.expectEq('foreach_array_access:zero', true, result.indexOf('zero') != -1 );
        Assert.expectEq('foreach_array_access:4', true, result.indexOf(4) != -1 );
        Assert.expectEq('foreach_array_access:netscape', true, result.indexOf('netscape') != -1 );
        Assert.expectEq('foreach_array_access:2000', true, result.indexOf(2000) != -1 );
    }

    public function for_prop_access(zam)
    {
        var result = []
        for ( super['moo'] in zam ) {
            result.push(super.moo);
        }
        Assert.expectEq('for_prop_access:length', true, result.indexOf('length') != -1 );
        Assert.expectEq('for_prop_access:company', true, result.indexOf('company') != -1 );
        Assert.expectEq('for_prop_access:year', true, result.indexOf('year') != -1 );
        Assert.expectEq('for_prop_access:0', true, result.indexOf(0) != -1 );
    }

    public function foreach_prop_access(zam)
    {
        var result = []
        for each ( super['moo'] in zam ) {
            result.push(super.moo);
        }
        Assert.expectEq('foreach_prop_access:zero', true, result.indexOf('zero') != -1 );
        Assert.expectEq('foreach_prop_access:4', true, result.indexOf(4) != -1 );
        Assert.expectEq('foreach_prop_access:netscape', true, result.indexOf('netscape') != -1 );
        Assert.expectEq('foreach_prop_access:2000', true, result.indexOf(2000) != -1 );
    }
}

// var SECTION = "Super";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";        // Version of ECMAScript or ActionScript
// var TITLE   = "Super used as assignment target in for loops";       // Provide ECMA section title or a description


var keys = ['length', 'company', 'year', 0];
var object = { length:4, company:"netscape", year:2000, 0:"zero" };
new Zonk().for_array_access(object);
new Zonk().foreach_array_access(object);
new Zonk().for_prop_access(object);
new Zonk().foreach_prop_access(object);


