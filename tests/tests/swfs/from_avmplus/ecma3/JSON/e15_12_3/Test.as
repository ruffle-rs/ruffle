/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;


function removeExceptionDetail(s:String) {
    var fnd=s.indexOf(" ");
    if (fnd>-1) {
        if (s.indexOf(':',fnd)>-1) {
                s=s.substring(0,s.indexOf(':',fnd));
        }
    }
    return s;
}

function sortObject(o:Object) {
    var keys=[];
    var key;
    for ( key in o ) {
        if (o[key]===undefined) {
           continue;
        }
        keys[keys.length]=key;
    }
    keys.sort();
    var ret="{";
    var value;
    for (var i in keys) {
        key=keys[i];
        value=o[key];
        if (value is String) {
            value='"'+value+'"';
        } else if (value is Array) {
            value='['+value+']';
        } else if (value is Object) {
        }
        ret += '"'+key+'":'+value+",";
    }
    ret=ret.substring(0,ret.length-1);
    ret+="}";
    return ret;
}

//package {

//     var SECTION = "15.2.3";
//     var VERSION = "ECMA_3";
//     var TITLE   = "JSON ecma-262 testcases";



    Assert.expectEq("15.12.3-0-1: JSON.stringify must exist as a function","function",typeof(JSON.stringify));
    Assert.expectEq("15.12.3-0-2: JSON.stringify must exist as a function taking 3 parameters",3,JSON.stringify.length);

    var exception1="no exception";
    try {
        JSON.stringify([42],{},'[42]');
    } catch (e) {
        exception1=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.3-4-1: JSON.stringify ignores replacer aruguments that are not functions or arrays..","TypeError: Error #1131",exception1);

    var obj = {a1: {b1: [1,2,3,4], b2: {c1: 1, c2: 2}},a2: 'a2'};
    Assert.expectEq("15.12.3-5-a-i-1: JSON.stringify converts Number wrapper object space aruguments to Number values", true, JSON.stringify(obj,null, new Number(5)) === JSON.stringify(obj,null,5));

    Assert.expectEq("15.12.3-5-b-i-1: JSON.stringify converts String wrapper object space aruguments to String values", true, JSON.stringify(obj,null, new String('xxx')) === JSON.stringify(obj,null, 'xxx'));

    Assert.expectEq("15.12.3-6-a-1: JSON.stringify treats numeric space arguments greater than 10 the same as a  space argument of 10.", true, JSON.stringify(obj,null, 10) === JSON.stringify(obj,null, 100));

    Assert.expectEq("15.12.3-6-a-2: JSON.stringify truccates non-integer numeric space arguments to their integer part.", true, JSON.stringify(obj,null, 5.99999) === JSON.stringify(obj,null, 5));

    Assert.expectEq("15.12.3-6-b-1: JSON.stringify treats numeric space arguments less than 1 (0.999999) the same as emptry string space argument.", true, JSON.stringify(obj,null, 0.999999) === JSON.stringify(obj));

    Assert.expectEq("15.12.3-6-b-2: JSON.stringify treats numeric space arguments less than 1 (0)the same as emptry string space argument.", true, JSON.stringify(obj,null, 0) === JSON.stringify(obj));

    Assert.expectEq("15.12.3-6-b-3: JSON.stringify treats numeric space arguments less than 1 (-5) the same as empty string space argument.", true, JSON.stringify(obj,null, -5) === JSON.stringify(obj));

    var fiveSpaces='     ';
    Assert.expectEq("15.12.3-6-b-4: JSON.stringify treats numeric space arguments (in the range 1..10) is equivalent to a string of spaces of that length.", true, JSON.stringify(obj,null, 5) === JSON.stringify(obj, null, fiveSpaces));

    Assert.expectEq("15.12.3-7-a-1: JSON.stringify only uses the first 10 characters of a string space arguments.", true, JSON.stringify(obj,null, '0123456789xxxxxxxxx')=== JSON.stringify(obj,null, '0123456789'));

    Assert.expectEq("15.12.3-8-a-1: JSON.stringify treats an empty string space argument the same as a missing space argument.", true, JSON.stringify(obj)=== JSON.stringify(obj,null, ''));
    try {
        Assert.expectEq("15.12.3-8-a-2: JSON.stringify treats an Boolean space argument the same as a missing space argument.", true, JSON.stringify(obj)=== JSON.stringify(obj,null, true));
    } catch (e) {
        Assert.expectEq("15.12.3-8-a-2: JSON.stringify should treat Boolean space argument same as missing space argument", 'no exception', e.toString());
    }

    try {
        Assert.expectEq("15.12.3-8-a-3: JSON.stringify treats an null space argument the same as a missing space argument.", true, JSON.stringify(obj)=== JSON.stringify(obj,null, null));
    } catch (e) {
        Assert.expectEq("15.12.3-8-a-3: JSON.stringify treats an null space argument the same as a missing space argument. '",'no exception',e.toString());
    }

    try {
        Assert.expectEq("15.12.3-8-a-4: JSON.stringify treats an Boolean wrapper space argument the same as a missing space argument.", true, JSON.stringify(obj)=== JSON.stringify(obj,null, new Boolean(true)));
    } catch (e) {
        Assert.expectEq("15.12.3-8-a-4: JSON.stringify treats an Boolean wrapper space argument the same as a missing space argument. '",'no exception',e.toString());
    }

    try {
        Assert.expectEq("15.12.3-8-a-5: JSON.stringify treats non-Number or String object space arguments the same as a missing space argument.", true, JSON.stringify(obj)=== JSON.stringify(obj,null, new Boolean(true)));
    } catch (e) {
        Assert.expectEq("15.12.3-8-a-5: JSON.stringify treats non-Number or String object space arguments the same as a missing space argument. '", 'no exception',e.toString());
    }

    var obj2 = {
        prop:42,
        toJSON: function () {return 'fortytwo objects'}
    };
    Assert.expectEq("15.12.3-2-2-b-i-1: JSON.stringify converts string wrapper objects returned from a toJSON call to literal strings.", '["fortytwo objects"]', JSON.stringify([obj2]));

    var obj3 = {
        prop:42,
        toJSON: function () {return new Number(42)}
    };
    Assert.expectEq("15.12.3-2-2-b-i-2: JSON.stringify converts Number wrapper objects returned from a toJSON call to literal Number.", '[42]', JSON.stringify([obj3]));

    var obj4 = {
        prop:42,
        toJSON: function () {return new Boolean(true)}
    };
    Assert.expectEq("15.12.3-2-2-b-i-3: JSON.stringify converts Boolean wrapper objects returned from a toJSON call to literal Boolean values.", '[true]', JSON.stringify([obj4]));

    // [fortytwo] ??
    Assert.expectEq("15.12.3-2-3-a-1: JSON.stringify converts string wrapper objects returned from replacer functions to literal strings.", '["fortytwo"]', JSON.stringify([42], function(k,v) {return v===42? new String('fortytwo'):v}));

    Assert.expectEq("15.12.3-2-3-a-2: JSON.stringify converts Number wrapper objects returned from replacer functions to literal numbers.", '[84]', JSON.stringify([42], function(k,v) {return v===42? new Number(84):v}));

    Assert.expectEq("15.12.3-2-3-a-3: JSON.stringify converts Boolean wrapper objects returned from replacer functions to literal numbers.", '[false]' ,JSON.stringify([42], function(k,v) {return v===42? new Boolean(false):v}));


    // See Bugzilla 654574 for info on tests 15.12.3-4-b-1 through 15.12.3-4-b-10

    Assert.expectEq("15.12.3-4-b-1: JSON.stringify replacer array has negative integer already converted to string.", '{"-1":18}', JSON.stringify({ "-0": 17, "-1": 18, 0: 19 }, ["-1"]));
    Assert.expectEq("15.12.3-4-b-2: JSON.stringify replacer array has negative integer not yet converted to string.", '{"-1":18}', JSON.stringify({ "-0": 17, "-1": 18, 0: 19 }, [-1]));
    Assert.expectEq("15.12.3-4-b-3: JSON.stringify replacer array has negative zero already converted to string.", '{"-0":17}', JSON.stringify({ "-0": 17, "-1": 18 , 0: 19 }, ["-0"]));
    Assert.expectEq("15.12.3-4-b-4: JSON.stringify replacer array has negative zero not yet converted to string.", '{"0":19}', JSON.stringify({ "-0": 17, "-1": 18 , 0: 19 }, [-0]));


    Assert.expectEq("15.12.3-4-b-5: JSON.stringify replacer array has double already converted to string.", '{"1.2":19}', JSON.stringify({ 1.2: 19 }, ["1.2"]));
    Assert.expectEq("15.12.3-4-b-6: JSON.stringify replacer array has double not yet converted to string.", '{"1.2":19}', JSON.stringify({ 1.2: 19 }, [1.2]));

    Assert.expectEq("15.12.3-4-b-7: JSON.stringify replacer array has repeated entries.", '{"a":1,"b":2}', JSON.stringify({ a: 1, b: 2, c: 3 }, ["a", "b", "b", "a"]));

    var objkey = {"key":3};
    Assert.expectEq("15.12.3-4-b-8: JSON.stringify replacer array has non-string non-numberentries.", '{"a":1,"b":2}', JSON.stringify({ a: 1, b: 2, c: 3 }, ["a", "b", objkey]));

    var gappy_array = [];
    gappy_array[0]   = 'a';
    gappy_array[100] = 'b';
    Assert.expectEq("15.12.3-4-b-9: JSON.stringify replacer array has gaps.", '{"a":1,"b":2}', JSON.stringify({ a: 1, b: 2, c: 3 }, gappy_array));

    Assert.expectEq("15.12.3-4-b-10: JSON.stringify replacer array nesting of keys.", '{"a":1,"b":{"b":3,"d":2}}', JSON.stringify({ a: 1, b: { d: 2, b: 3 }, c: 4 }, ["a", "b", "d"]));


// }
