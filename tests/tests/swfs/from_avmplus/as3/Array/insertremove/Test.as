/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

/**
*   File Name:    insertremove.as
*   Description:  Test Array.insertAt() and Array.removeAt().  These are AS3 extensions defined in terms of ES3 Array.splice().
*/

// var SECTION="";
// var VERSION = "ECMA_1";


function CheckInsertAt(arr, isSparse, index, element)
{
	var description = "insert into " + arr.length + "-element " + (isSparse ? "sparse" : "dense") + " array at " + index;

	var arr1 = arr.concat(); // shallow copy

	arr1.splice(index, 0, element);

	Assert.expectEq(description + ": length after splice", arr1.length, arr.length+1);

	var arr2 = arr.concat(); // shallow copy

	arr2.insertAt(index, element);

	Assert.expectEq(description + ": length after insertAt", arr2.length, arr.length+1);

	for (var i = 0; i < arr.length+1; i++)
	{
		Assert.expectEq(description + ": element " + i, arr1[i], arr2[i]);
	}
}

function CheckRemoveAt(arr, isSparse, index)
{
	var description = "remove from " + arr.length + "-element " + (isSparse ? "sparse" : "dense") + " array at " + index;

	var arr1 = arr.concat(); // shallow copy

	var element1 = arr1.splice(index, 1)[0];

	var arr2 = arr.concat(); // shallow copy

	var element2 = arr2.removeAt(index);

	Assert.expectEq(description + ": length", arr1.length, arr2.length);

	Assert.expectEq(description + ": result", element1, element2);

	var count = arr1.length-1;
	if (count > arr2.length-1) count = arr2.length-1;

	for (var i = 0; i < count; i++)
	{
		Assert.expectEq(description + ": element " + i, arr1[i], arr2[i]);
	}
}

function CheckArray(arr, isSparse, first, count, stride)
{
	count++; // go one extra for better coverage

	for (var i = 0; i <= count; i++)
	{
		var index = (first + i) * stride;

		if (i == 0 || stride != 1)
		{
			CheckInsertAt(arr, isSparse, index-1, "foo");
			CheckRemoveAt(arr, isSparse, index-1);
		}

		CheckInsertAt(arr, isSparse, index,   "foo");
		CheckRemoveAt(arr, isSparse, index);

		if (i == count || stride != 1)
		{
			CheckInsertAt(arr, isSparse, index+1, "foo");
			CheckRemoveAt(arr, isSparse, index+1);
		}

		index = -index;

		if (i == 0 || stride != 1)
		{
			CheckInsertAt(arr, isSparse, index+1, "foo");
			CheckRemoveAt(arr, isSparse, index+1);
		}

		CheckInsertAt(arr, isSparse, index,   "foo");
		CheckRemoveAt(arr, isSparse, index);

		if (i == count || stride != 1)
		{
			CheckInsertAt(arr, isSparse, index-1, "foo");
			CheckRemoveAt(arr, isSparse, index-1);
		}
	}
}


// Dense arrays (m_denseStart == 0).

var arr0 = [ ];  // All empty arrays are dense.

CheckArray(arr0, false, 0, 0, 1);

var arr1 = ["one"];

CheckArray(arr1, false, 0, 1, 1);

var arr2 = ["one", "two"];

CheckArray(arr2, false, 0, 2, 1);

var arr3 = ["one", "two", "three"];

CheckArray(arr3, false, 0, 3, 1);


// Dense arrays (m_denseStart != 0).

var d_arr1 = new Array();
d_arr1[5] = "one";

CheckArray(d_arr1, false, 5, 1, 1);

var d_arr2 = new Array();
d_arr2[5] = "one";
d_arr2[6] = "two";

CheckArray(d_arr2, false, 5, 2, 1);

var d_arr3 = new Array();
d_arr3[5] = "one";
d_arr3[6] = "two";
d_arr3[7] = "three";

CheckArray(d_arr3, false, 5, 3, 1);


// Sparse arrays.

var s_arr1 = new Array();
s_arr1[100] = "one";   // Array starts off dense.
s_arr1[200] = "";      // This assignment converts it to sparse.
delete s_arr1[200];    // Sparse arrays never revert to dense unless empty
// We now have a sparse array with a single entry at index 100.

CheckArray(s_arr1, true, 0, 1, 100);

var s_arr2 = new Array();
s_arr2[100] = "one";
s_arr2[200] = "two";

CheckArray(s_arr2, true, 0, 2, 100);

var s_arr3 = new Array();
s_arr3[100] = "one";
s_arr3[200] = "two";
s_arr3[200] = "three";

CheckArray(s_arr3, true, 0, 3, 100);


// Dense array with holes.

Array.prototype[5] = "proto_five";
Array.prototype[8] = "proto_eight";

var h_arr1 = new Array();

h_arr1[2] = "two";
h_arr1[3] = "three";
h_arr1[8] = "eight";
h_arr1[10] = "ten";

CheckArray(h_arr1, false, 0, h_arr1.length, 1);


