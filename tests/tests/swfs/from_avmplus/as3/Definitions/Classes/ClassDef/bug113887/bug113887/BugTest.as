/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// bug 113887: static initialization using the class itself
package bug113887 {
    public class BugTest {
        public function doBasicTest() : String {
            return A.obj.wasInitialized();
        }
        public function doFunctionTest() : String {
            return B.foo.wasInitialized();
        }
    }
    
}

