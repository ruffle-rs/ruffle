/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "instanceof-002";
//     var VERSION = "ECMA_2";
//     var TITLE   = "Determining Instance Relationships";


    var testcases = getTestCases();

function getTestCases() {
        var array = new Array();
        var item = 0;
        
    function Employee ( name, dept ) {
         this.name = name || "";
         this.dept = dept || "general";
    }
    
    function Manager () {
         this.reports = [];
    }
    Manager.prototype = new Employee();
    
    function WorkerBee ( name, dept, projs ) {
        this.base = Employee;
        this.base( name, dept)
        this.projects = projs || new Array();
    }
    WorkerBee.prototype = new Employee();
    
    function SalesPerson () {
        this.dept = "sales";
        this.quota = 100;
    }
    SalesPerson.prototype = new WorkerBee();
    
    function Engineer ( name, projs, machine ) {
        this.base = WorkerBee;
        this.base( name, "engineering", projs )
        this.machine = machine || "";
    }
    Engineer.prototype = new WorkerBee();
    
    var pat = new Engineer()


    array[item++] = Assert.expectEq( 
                                    "pat.constructor.prototype == Engineer.prototype",
                                    false,
                                    pat.constructor.prototype == Engineer.prototype );

    array[item++] = Assert.expectEq( 
                                    "pat instanceof Engineer",
                                    true,
                                    pat instanceof Engineer );

    array[item++] = Assert.expectEq( 
                                    "pat instanceof WorkerBee )",
                                    true,
                                     pat instanceof WorkerBee );

    array[item++] = Assert.expectEq( 
                                    "pat instanceof Employee )",
                                    true,
                                     pat instanceof Employee );

    array[item++] = Assert.expectEq( 
                                    "pat instanceof Object )",
                                    true,
                                     pat instanceof Object );

    array[item++] = Assert.expectEq( 
                                    "pat instanceof SalesPerson )",
                                    false,
                                     pat instanceof SalesPerson );
    return array;
}
