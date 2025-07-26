/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "regress_723461";
// var VERSION = "AS3";
// var TITLE   = "associative Array defined with [] does not show non-index elements as propertyIsEnumerable";
// var bug = "723461";


var emptyA = [];
var denseZ = [];
var denseO = [];
var indexK = [];
var sparse = [];
var object = {};

var all = [emptyA, denseZ, denseO, indexK, sparse, object];

denseZ[0]    = "dense starting at zero";

denseO[1]    = "dense starting at one";

indexK[1000] = "array starting at thousand";

sparse[1]    = "array starting at one";
sparse[1000] = "assuredly sparse due to element at thousand";

object[1000] = "object with value at thousand";

for each (x in all) {
    x["nonindex"] = "a non index in every tested object";
}

function AddCleanCase(name, object, key, expect) {
    Assert.expectEq(name, expect, object.propertyIsEnumerable(key));
}

AddCleanCase("emptyA pIE on present nonindex",   emptyA, "nonindex", true);
AddCleanCase("emptyA pIE on absent nonindex",    emptyA, "absent",  false);
AddCleanCase("denseZ pIE on present nonindex",   denseZ, "nonindex", true);
AddCleanCase("denseZ pIE on absent nonindex",    denseZ, "absent",  false);
AddCleanCase("denseO pIE on present nonindex",   denseO, "nonindex", true);
AddCleanCase("denseO pIE on absent nonindex",    denseO, "absent",  false);
AddCleanCase("indexK pIE on present nonindex",   indexK, "nonindex", true);
AddCleanCase("indexK pIE on absent nonindex",    indexK, "absent",  false);
AddCleanCase("sparse pIE on present nonindex",   sparse, "nonindex", true);
AddCleanCase("sparse pIE on absent nonindex",    sparse, "absent",  false);
AddCleanCase("object pIE on present nonindex",   object, "nonindex", true);
AddCleanCase("object pIE on absent nonindex",    object, "absent",  false);

function AddToggleCase(name, object, key, expect) {
    object.setPropertyIsEnumerable(key, false);
    Assert.expectEq(name, expect, object.propertyIsEnumerable(key));
}

AddToggleCase("emptyA tog pIE present nonindex", emptyA, "nonindex", false);
AddToggleCase("emptyA tog pIE absent nonindex",  emptyA, "absent",   false);
AddToggleCase("denseZ tog pIE present nonindex", denseZ, "nonindex", false);
AddToggleCase("denseZ tog pIE absent nonindex",  denseZ, "absent",   false);
AddToggleCase("denseO tog pIE present nonindex", denseO, "nonindex", false);
AddToggleCase("denseO tog pIE absent nonindex",  denseO, "absent",   false);
AddToggleCase("indexK tog pIE present nonindex", indexK, "nonindex", false);

AddToggleCase("indexK tog pIE absent nonindex",  indexK, "absent",   false);
AddToggleCase("sparse tog pIE present nonindex", sparse, "nonindex", false);
AddToggleCase("sparse tog pIE absent nonindex",  sparse, "absent",   false);
AddToggleCase("object tog pIE present nonindex", object, "nonindex", false);
AddToggleCase("object tog pIE absent nonindex",  object, "absent",   false);
