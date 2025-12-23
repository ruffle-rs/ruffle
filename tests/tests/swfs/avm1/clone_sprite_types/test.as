trace("Timeline objects:");
trace(timeline_button);
trace(timeline_edittext);
trace(timeline_statictext);
trace(timeline_sprite);
trace(timeline_shape);
trace(timeline_morphshape);
trace(timeline_image);
trace(timeline_video);

duplicateMovieClip(timeline_button, "timeline_button_2", 101);
duplicateMovieClip(timeline_edittext, "timeline_edittext_2", 102);
duplicateMovieClip(timeline_statictext, "timeline_statictext_2", 103);
duplicateMovieClip(timeline_sprite, "timeline_sprite_2", 104);
duplicateMovieClip(timeline_shape, "timeline_shape_2", 105);
duplicateMovieClip(timeline_morphshape, "timeline_morphshape_2", 106);
duplicateMovieClip(timeline_image, "timeline_image_2", 107);
duplicateMovieClip(timeline_video, "timeline_video_2", 108);

trace("Timeline clones:");
trace(timeline_button_2);
trace(timeline_edittext_2);
trace(timeline_statictext_2);
trace(timeline_sprite_2);
trace(timeline_shape_2);
trace(timeline_morphshape_2);
trace(timeline_image_2);
trace(timeline_video_2);

createEmptyMovieClip("script_movieclip", 201);
createTextField("script_textfield", 202, 0, 0, 10, 10);

trace("Script objects:");
trace(script_movieclip);
trace(script_textfield);

duplicateMovieClip(script_movieclip, "script_movieclip_2", 301);
duplicateMovieClip(script_textfield, "script_textfield_2", 302);

trace("Script clones:");
trace(script_movieclip_2);
trace(script_textfield_2);
