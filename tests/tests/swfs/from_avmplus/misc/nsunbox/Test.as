/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

namespace foons = "foo";

function ns_to_string(ns:Namespace) { return String(ns); }


Assert.expectEq("ns_to_string(foons)", "foo", ns_to_string(foons));
Assert.expectEq("ns_to_string(null)", "null", ns_to_string(null));
Assert.expectEq("ns_to_string(void 0)", "null", ns_to_string(void 0)); // unbox undefined -> Namespace should yield null


