_root.createTextField("tf", 1, 0, 0, 400, 80);
_root.tf.html = true;
_root.tf.htmlText = '<font size="32"><a href="AsFuNcTiOn:test">CLICKME</a></font>';
_global.test = function() { trace("called"); };
