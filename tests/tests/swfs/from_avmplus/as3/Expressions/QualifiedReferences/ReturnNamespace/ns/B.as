/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package ns {

    public class B {
        public function befriendAnA(a:A) {
        var key:Namespace = a.beMyFriend(this)
        return a.key::makeMyDay();
        
        }
    }

}
