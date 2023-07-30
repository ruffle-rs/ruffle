/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = 'As described in Netscape doc "Whats new in JavaScript 1.2"';
//     var VERSION = 'no version';
//     var TITLE = 'String:push,splice,concat,unshift,sort';


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var array1 = new Array();
    var array2 = [];
    var array5 = [0,1,2,3,4,5,6,7,8,9];
    var array6 = [9,8,7,6,5,4,3,2,1,0];
    var size   = 10;

        for (var i=0; i < size; i++)
        {
            array1[i] = i;
            array2[i] = size-1-i;
        }


    // the following for loop reverses the order of array1 so
    // that it should be similarly ordered to array2
    var array3;
    for (i = array1.length; i > 0; i--)
    {
        array3 = array1.slice(1,i);
        array1.splice(1,i-1);
        array1 = array3.concat(array1);
    }

        // the following for loop reverses the order of array1 so
    // that it should be similarly ordered to array2
    var array3;
    for (i = array5.length; i > 0; i--)
    {
        array7 = array5.slice(1,i);
        array5.splice(1,i-1);
        array5 = array7.concat(array5);
    }

    // the following for loop reverses the order of array1
    // and array2
    for (i = 0; i < size; i++)
    {
        array1.push(array1.shift());
        array2.unshift(array2.pop());
    }

        // the following for loop reverses the order of array5
    // and array6
    for (i = 0; i < size; i++)
    {
        array5.push(array5.shift());
        array6.unshift(array6.pop());
    }

    array[item++] = Assert.expectEq(  "Array.push,pop,shift,unshift,slice,splice", true,String(array1) == String(array2));
        array[item++] = Assert.expectEq(  "Array.push,pop,shift,unshift,slice,splice", true,String(array5) == String(array6));
    array1.sort();
    array2.sort();
        array5.sort();
        array6.sort();
    array[item++] = Assert.expectEq(  "Array.sort", true,String(array1) == String(array2));
        array[item++] = Assert.expectEq(  "Array.sort", true,String(array5) == String(array6));

    return ( array );
}
