/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
 
 
 
 package DynamicClassHasOwnPropertyPackage {
 
    
   public dynamic class DynamicClassHasOwnProperty {
   
       public var bar:Number;
       public function testHasOwnProperty(p:String) {
        bar = 10;
        return this.hasOwnProperty(p);
       }

   }
   
}
