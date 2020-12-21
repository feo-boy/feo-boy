/******/ (function(modules) { // webpackBootstrap
/******/ 	// install a JSONP callback for chunk loading
/******/ 	function webpackJsonpCallback(data) {
/******/ 		var chunkIds = data[0];
/******/ 		var moreModules = data[1];
/******/
/******/
/******/ 		// add "moreModules" to the modules object,
/******/ 		// then flag all "chunkIds" as loaded and fire callback
/******/ 		var moduleId, chunkId, i = 0, resolves = [];
/******/ 		for(;i < chunkIds.length; i++) {
/******/ 			chunkId = chunkIds[i];
/******/ 			if(Object.prototype.hasOwnProperty.call(installedChunks, chunkId) && installedChunks[chunkId]) {
/******/ 				resolves.push(installedChunks[chunkId][0]);
/******/ 			}
/******/ 			installedChunks[chunkId] = 0;
/******/ 		}
/******/ 		for(moduleId in moreModules) {
/******/ 			if(Object.prototype.hasOwnProperty.call(moreModules, moduleId)) {
/******/ 				modules[moduleId] = moreModules[moduleId];
/******/ 			}
/******/ 		}
/******/ 		if(parentJsonpFunction) parentJsonpFunction(data);
/******/
/******/ 		while(resolves.length) {
/******/ 			resolves.shift()();
/******/ 		}
/******/
/******/ 	};
/******/
/******/
/******/ 	// The module cache
/******/ 	var installedModules = {};
/******/
/******/ 	// object to store loaded and loading chunks
/******/ 	// undefined = chunk not loaded, null = chunk preloaded/prefetched
/******/ 	// Promise = chunk loading, 0 = chunk loaded
/******/ 	var installedChunks = {
/******/ 		"main": 0
/******/ 	};
/******/
/******/
/******/
/******/ 	// script path function
/******/ 	function jsonpScriptSrc(chunkId) {
/******/ 		return __webpack_require__.p + "" + chunkId + ".index.js"
/******/ 	}
/******/
/******/ 	// object to store loaded and loading wasm modules
/******/ 	var installedWasmModules = {};
/******/
/******/ 	function promiseResolve() { return Promise.resolve(); }
/******/
/******/ 	var wasmImportObjects = {
/******/ 		"./pkg/index_bg.wasm": function() {
/******/ 			return {
/******/ 				"./index_bg.js": {
/******/ 					"__wbindgen_object_clone_ref": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_object_clone_ref"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_cb_drop": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_cb_drop"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_string_new": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_string_new"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_error_4bb6c2a97407129a": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_error_4bb6c2a97407129a"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_new_59cb74e423758ede": function() {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_new_59cb74e423758ede"]();
/******/ 					},
/******/ 					"__wbg_stack_558ba5917b466edd": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_stack_558ba5917b466edd"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_object_drop_ref": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_object_drop_ref"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_number_new": function(p0f64) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_number_new"](p0f64);
/******/ 					},
/******/ 					"__wbg_instanceof_Window_49f532f06a9786ee": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_instanceof_Window_49f532f06a9786ee"](p0i32);
/******/ 					},
/******/ 					"__wbg_document_c0366b39e4f4c89a": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_document_c0366b39e4f4c89a"](p0i32);
/******/ 					},
/******/ 					"__wbg_navigator_95ba9cd684cf90aa": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_navigator_95ba9cd684cf90aa"](p0i32);
/******/ 					},
/******/ 					"__wbg_innerWidth_cea04a991524ea87": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_innerWidth_cea04a991524ea87"](p0i32);
/******/ 					},
/******/ 					"__wbg_innerHeight_83651dca462998d1": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_innerHeight_83651dca462998d1"](p0i32);
/******/ 					},
/******/ 					"__wbg_devicePixelRatio_268c49438a600d53": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_devicePixelRatio_268c49438a600d53"](p0i32);
/******/ 					},
/******/ 					"__wbg_cancelAnimationFrame_60f9cf59ec1c0125": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_cancelAnimationFrame_60f9cf59ec1c0125"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_matchMedia_f9355258d56dc891": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_matchMedia_f9355258d56dc891"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_requestAnimationFrame_ef0e2294dc8b1088": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_requestAnimationFrame_ef0e2294dc8b1088"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_get_03d057a4fd2b7031": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_get_03d057a4fd2b7031"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_clearTimeout_cf42c747400433ba": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_clearTimeout_cf42c747400433ba"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_setTimeout_7df13099c62f73a7": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_setTimeout_7df13099c62f73a7"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_target_4bc4eb28204bcc44": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_target_4bc4eb28204bcc44"](p0i32);
/******/ 					},
/******/ 					"__wbg_cancelBubble_62eb67fd286e013f": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_cancelBubble_62eb67fd286e013f"](p0i32);
/******/ 					},
/******/ 					"__wbg_preventDefault_9aab6c264e5df3ee": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_preventDefault_9aab6c264e5df3ee"](p0i32);
/******/ 					},
/******/ 					"__wbg_stopPropagation_697200010cec9b7e": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_stopPropagation_697200010cec9b7e"](p0i32);
/******/ 					},
/******/ 					"__wbg_configureSwapChain_7f621c56f3688ff3": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_configureSwapChain_7f621c56f3688ff3"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_instanceof_HtmlCanvasElement_7bd3ee7838f11fc3": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_instanceof_HtmlCanvasElement_7bd3ee7838f11fc3"](p0i32);
/******/ 					},
/******/ 					"__wbg_width_0efa4604d41c58c5": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_width_0efa4604d41c58c5"](p0i32);
/******/ 					},
/******/ 					"__wbg_setwidth_1d0e975feecff3ef": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_setwidth_1d0e975feecff3ef"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_height_aa24e3fef658c4a8": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_height_aa24e3fef658c4a8"](p0i32);
/******/ 					},
/******/ 					"__wbg_setheight_7758ee3ff5c65474": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_setheight_7758ee3ff5c65474"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_getContext_3db9399e6dc524ff": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_getContext_3db9399e6dc524ff"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_label_800a26e9d25ddd13": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_label_800a26e9d25ddd13"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_beginRenderPass_259abc2be17aa112": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_beginRenderPass_259abc2be17aa112"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_finish_7e138b0949c43e47": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_finish_7e138b0949c43e47"](p0i32);
/******/ 					},
/******/ 					"__wbg_finish_004ebbefd95c7f72": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_finish_004ebbefd95c7f72"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_matches_2f8453eb8e607f46": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_matches_2f8453eb8e607f46"](p0i32);
/******/ 					},
/******/ 					"__wbg_addListener_34d9bdd94b12c993": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_addListener_34d9bdd94b12c993"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_removeListener_5571e3bc24e85d2c": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_removeListener_5571e3bc24e85d2c"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_x_d61460e3c817f5b2": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_x_d61460e3c817f5b2"](p0i32);
/******/ 					},
/******/ 					"__wbg_y_e4e5b87d074dc33d": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_y_e4e5b87d074dc33d"](p0i32);
/******/ 					},
/******/ 					"__wbg_gpu_a95ac4bfa3eeb7aa": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_gpu_a95ac4bfa3eeb7aa"](p0i32);
/******/ 					},
/******/ 					"__wbg_submit_ef69ce25829dcba1": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_submit_ef69ce25829dcba1"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_writeBuffer_4e96382296e35093": function(p0i32,p1i32,p2f64,p3i32,p4f64,p5f64) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_writeBuffer_4e96382296e35093"](p0i32,p1i32,p2f64,p3i32,p4f64,p5f64);
/******/ 					},
/******/ 					"__wbg_writeTexture_91f0b9f56d05cfd5": function(p0i32,p1i32,p2i32,p3i32,p4i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_writeTexture_91f0b9f56d05cfd5"](p0i32,p1i32,p2i32,p3i32,p4i32);
/******/ 					},
/******/ 					"__wbg_setProperty_46b9bd1b0fad730b": function(p0i32,p1i32,p2i32,p3i32,p4i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_setProperty_46b9bd1b0fad730b"](p0i32,p1i32,p2i32,p3i32,p4i32);
/******/ 					},
/******/ 					"__wbg_requestAdapter_3b9bf99cdc065385": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_requestAdapter_3b9bf99cdc065385"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_endPass_e09b640346ed3592": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_endPass_e09b640346ed3592"](p0i32);
/******/ 					},
/******/ 					"__wbg_setBindGroup_78ade7caea0ec878": function(p0i32,p1i32,p2i32,p3i32,p4i32,p5f64,p6i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_setBindGroup_78ade7caea0ec878"](p0i32,p1i32,p2i32,p3i32,p4i32,p5f64,p6i32);
/******/ 					},
/******/ 					"__wbg_draw_82f4a8a7bde2e02c": function(p0i32,p1i32,p2i32,p3i32,p4i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_draw_82f4a8a7bde2e02c"](p0i32,p1i32,p2i32,p3i32,p4i32);
/******/ 					},
/******/ 					"__wbg_setPipeline_86bdb4403d832dc0": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_setPipeline_86bdb4403d832dc0"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_createView_3e6f4309074d79dc": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_createView_3e6f4309074d79dc"](p0i32);
/******/ 					},
/******/ 					"__wbg_createView_50bb39a4a775d0cc": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_createView_50bb39a4a775d0cc"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_clientX_3a14a1583294607f": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_clientX_3a14a1583294607f"](p0i32);
/******/ 					},
/******/ 					"__wbg_clientY_4b4a322b80551002": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_clientY_4b4a322b80551002"](p0i32);
/******/ 					},
/******/ 					"__wbg_offsetX_4bd8c9fcb457cf0b": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_offsetX_4bd8c9fcb457cf0b"](p0i32);
/******/ 					},
/******/ 					"__wbg_offsetY_0dde12490e8ebfba": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_offsetY_0dde12490e8ebfba"](p0i32);
/******/ 					},
/******/ 					"__wbg_ctrlKey_fadbf4d226c5a071": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_ctrlKey_fadbf4d226c5a071"](p0i32);
/******/ 					},
/******/ 					"__wbg_shiftKey_6df8deff50c0048c": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_shiftKey_6df8deff50c0048c"](p0i32);
/******/ 					},
/******/ 					"__wbg_altKey_470315032c1b4a35": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_altKey_470315032c1b4a35"](p0i32);
/******/ 					},
/******/ 					"__wbg_metaKey_42ae5f8d628a98d5": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_metaKey_42ae5f8d628a98d5"](p0i32);
/******/ 					},
/******/ 					"__wbg_button_9e74bd912190b055": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_button_9e74bd912190b055"](p0i32);
/******/ 					},
/******/ 					"__wbg_buttons_5d3db1e47542f585": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_buttons_5d3db1e47542f585"](p0i32);
/******/ 					},
/******/ 					"__wbg_deltaX_5fac4f36a42e6ec9": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_deltaX_5fac4f36a42e6ec9"](p0i32);
/******/ 					},
/******/ 					"__wbg_deltaY_2722120e563d3160": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_deltaY_2722120e563d3160"](p0i32);
/******/ 					},
/******/ 					"__wbg_deltaMode_3db3c9c4bedf191d": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_deltaMode_3db3c9c4bedf191d"](p0i32);
/******/ 					},
/******/ 					"__wbg_fullscreenElement_40ed1ecabc8c860a": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_fullscreenElement_40ed1ecabc8c860a"](p0i32);
/******/ 					},
/******/ 					"__wbg_createElement_99351c8bf0efac6e": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_createElement_99351c8bf0efac6e"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_exitFullscreen_5cd6f888225ba968": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_exitFullscreen_5cd6f888225ba968"](p0i32);
/******/ 					},
/******/ 					"__wbg_getElementById_15aef17a620252b4": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_getElementById_15aef17a620252b4"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_querySelectorAll_51ffae19a5675fef": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_querySelectorAll_51ffae19a5675fef"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_setinnerHTML_79084edd97462c07": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_setinnerHTML_79084edd97462c07"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_getBoundingClientRect_505844bd8eb35668": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_getBoundingClientRect_505844bd8eb35668"](p0i32);
/******/ 					},
/******/ 					"__wbg_requestFullscreen_60b4644a038d0689": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_requestFullscreen_60b4644a038d0689"](p0i32);
/******/ 					},
/******/ 					"__wbg_setAttribute_e71b9086539f06a1": function(p0i32,p1i32,p2i32,p3i32,p4i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_setAttribute_e71b9086539f06a1"](p0i32,p1i32,p2i32,p3i32,p4i32);
/******/ 					},
/******/ 					"__wbg_setPointerCapture_54ee987062d42d03": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_setPointerCapture_54ee987062d42d03"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_getMappedRange_ef26c08d26631953": function(p0i32,p1f64,p2f64) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_getMappedRange_ef26c08d26631953"](p0i32,p1f64,p2f64);
/******/ 					},
/******/ 					"__wbg_unmap_f6decf13d06c307d": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_unmap_f6decf13d06c307d"](p0i32);
/******/ 					},
/******/ 					"__wbg_defaultQueue_9b3ffbc22c704fd3": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_defaultQueue_9b3ffbc22c704fd3"](p0i32);
/******/ 					},
/******/ 					"__wbg_createBindGroup_7400903fb0236c60": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_createBindGroup_7400903fb0236c60"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_createBindGroupLayout_9f92f0c82b00cd97": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_createBindGroupLayout_9f92f0c82b00cd97"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_createBuffer_b0a89a74f115179f": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_createBuffer_b0a89a74f115179f"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_createCommandEncoder_b49fea6ef6f6f3ac": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_createCommandEncoder_b49fea6ef6f6f3ac"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_createPipelineLayout_703131f2e452a409": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_createPipelineLayout_703131f2e452a409"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_createRenderPipeline_d69f49b960de7819": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_createRenderPipeline_d69f49b960de7819"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_createSampler_f4145fa0c4c49346": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_createSampler_f4145fa0c4c49346"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_createShaderModule_3f60f5b63b309185": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_createShaderModule_3f60f5b63b309185"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_createTexture_d98109ccd4f3e0cc": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_createTexture_d98109ccd4f3e0cc"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_matches_c1680f96c1f19da4": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_matches_c1680f96c1f19da4"](p0i32);
/******/ 					},
/******/ 					"__wbg_pointerId_602db5c989b38cc0": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_pointerId_602db5c989b38cc0"](p0i32);
/******/ 					},
/******/ 					"__wbg_debug_146b863607d79e9d": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_debug_146b863607d79e9d"](p0i32);
/******/ 					},
/******/ 					"__wbg_error_e325755affc8634b": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_error_e325755affc8634b"](p0i32);
/******/ 					},
/******/ 					"__wbg_error_d58d9958868010f6": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_error_d58d9958868010f6"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_info_d60054f760c729cc": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_info_d60054f760c729cc"](p0i32);
/******/ 					},
/******/ 					"__wbg_log_f2e13ca55da8bad3": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_log_f2e13ca55da8bad3"](p0i32);
/******/ 					},
/******/ 					"__wbg_warn_9e92ccdc67085e1b": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_warn_9e92ccdc67085e1b"](p0i32);
/******/ 					},
/******/ 					"__wbg_now_7628760b7b640632": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_now_7628760b7b640632"](p0i32);
/******/ 					},
/******/ 					"__wbg_instanceof_HtmlElement_ed44c8f443dbd619": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_instanceof_HtmlElement_ed44c8f443dbd619"](p0i32);
/******/ 					},
/******/ 					"__wbg_style_9b773f0fc441eddc": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_style_9b773f0fc441eddc"](p0i32);
/******/ 					},
/******/ 					"__wbg_addEventListener_6a37bc32387cb66d": function(p0i32,p1i32,p2i32,p3i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_addEventListener_6a37bc32387cb66d"](p0i32,p1i32,p2i32,p3i32);
/******/ 					},
/******/ 					"__wbg_addEventListener_a422088e686210b5": function(p0i32,p1i32,p2i32,p3i32,p4i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_addEventListener_a422088e686210b5"](p0i32,p1i32,p2i32,p3i32,p4i32);
/******/ 					},
/******/ 					"__wbg_removeEventListener_70dfb387da1982ac": function(p0i32,p1i32,p2i32,p3i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_removeEventListener_70dfb387da1982ac"](p0i32,p1i32,p2i32,p3i32);
/******/ 					},
/******/ 					"__wbg_requestDevice_9410154bd480aec7": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_requestDevice_9410154bd480aec7"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_getCurrentTexture_4f3bf6c3a30da5c1": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_getCurrentTexture_4f3bf6c3a30da5c1"](p0i32);
/******/ 					},
/******/ 					"__wbg_charCode_eb123e299efafe3f": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_charCode_eb123e299efafe3f"](p0i32);
/******/ 					},
/******/ 					"__wbg_keyCode_47f9e9228bc483bf": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_keyCode_47f9e9228bc483bf"](p0i32);
/******/ 					},
/******/ 					"__wbg_altKey_8a59e1cf32636010": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_altKey_8a59e1cf32636010"](p0i32);
/******/ 					},
/******/ 					"__wbg_ctrlKey_17377b46ca5a072d": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_ctrlKey_17377b46ca5a072d"](p0i32);
/******/ 					},
/******/ 					"__wbg_shiftKey_09be9a7e6cad7a99": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_shiftKey_09be9a7e6cad7a99"](p0i32);
/******/ 					},
/******/ 					"__wbg_metaKey_a707288e6c45a0e0": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_metaKey_a707288e6c45a0e0"](p0i32);
/******/ 					},
/******/ 					"__wbg_key_d9b602f48baca7bc": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_key_d9b602f48baca7bc"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_code_cbf76ad384ae1179": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_code_cbf76ad384ae1179"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_get_20fb2ed3ba07d2ee": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_get_20fb2ed3ba07d2ee"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_new_9dff83a08f5994f3": function() {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_new_9dff83a08f5994f3"]();
/******/ 					},
/******/ 					"__wbg_push_3ddd8187ff2ff82d": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_push_3ddd8187ff2ff82d"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_newnoargs_7c6bd521992b4022": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_newnoargs_7c6bd521992b4022"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_call_951bd0c6d815d6f1": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_call_951bd0c6d815d6f1"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_is_049b1aece40b5301": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_is_049b1aece40b5301"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_new_ba07d0daa0e4677e": function() {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_new_ba07d0daa0e4677e"]();
/******/ 					},
/******/ 					"__wbg_resolve_6e61e640925a0db9": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_resolve_6e61e640925a0db9"](p0i32);
/******/ 					},
/******/ 					"__wbg_then_dd3785597974798a": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_then_dd3785597974798a"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_then_0f957e0f4c3e537a": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_then_0f957e0f4c3e537a"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_globalThis_513fb247e8e4e6d2": function() {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_globalThis_513fb247e8e4e6d2"]();
/******/ 					},
/******/ 					"__wbg_self_6baf3a3aa7b63415": function() {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_self_6baf3a3aa7b63415"]();
/******/ 					},
/******/ 					"__wbg_window_63fc4027b66c265b": function() {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_window_63fc4027b66c265b"]();
/******/ 					},
/******/ 					"__wbg_global_b87245cd886d7113": function() {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_global_b87245cd886d7113"]();
/******/ 					},
/******/ 					"__wbg_new_c6c0228e6d22a2f9": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_new_c6c0228e6d22a2f9"](p0i32);
/******/ 					},
/******/ 					"__wbg_newwithbyteoffsetandlength_4c51342f87299c5a": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_newwithbyteoffsetandlength_4c51342f87299c5a"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_buffer_c385539cb4060297": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_buffer_c385539cb4060297"](p0i32);
/******/ 					},
/******/ 					"__wbg_length_c645e7c02233b440": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_length_c645e7c02233b440"](p0i32);
/******/ 					},
/******/ 					"__wbg_set_b91afac9fd216d99": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_set_b91afac9fd216d99"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_new_8f59c88fa4234f01": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_new_8f59c88fa4234f01"](p0i32);
/******/ 					},
/******/ 					"__wbg_newwithbyteoffsetandlength_2016b902c412c87c": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_newwithbyteoffsetandlength_2016b902c412c87c"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_buffer_3f12a1c608c6d04e": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_buffer_3f12a1c608c6d04e"](p0i32);
/******/ 					},
/******/ 					"__wbg_get_85e0a3b459845fe2": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_get_85e0a3b459845fe2"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_set_9bdd413385146137": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbg_set_9bdd413385146137"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbindgen_is_undefined": function(p0i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_is_undefined"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_number_get": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_number_get"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_debug_string": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_debug_string"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_throw": function(p0i32,p1i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_throw"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_memory": function() {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_memory"]();
/******/ 					},
/******/ 					"__wbindgen_closure_wrapper1843": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_closure_wrapper1843"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbindgen_closure_wrapper1845": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_closure_wrapper1845"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbindgen_closure_wrapper1847": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_closure_wrapper1847"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbindgen_closure_wrapper1849": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_closure_wrapper1849"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbindgen_closure_wrapper1851": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_closure_wrapper1851"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbindgen_closure_wrapper1853": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_closure_wrapper1853"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbindgen_closure_wrapper1855": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_closure_wrapper1855"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbindgen_closure_wrapper1857": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_closure_wrapper1857"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbindgen_closure_wrapper1859": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_closure_wrapper1859"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbindgen_closure_wrapper6765": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./pkg/index_bg.js"].exports["__wbindgen_closure_wrapper6765"](p0i32,p1i32,p2i32);
/******/ 					}
/******/ 				}
/******/ 			};
/******/ 		},
/******/ 	};
/******/
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/
/******/ 		// Check if module is in cache
/******/ 		if(installedModules[moduleId]) {
/******/ 			return installedModules[moduleId].exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = installedModules[moduleId] = {
/******/ 			i: moduleId,
/******/ 			l: false,
/******/ 			exports: {}
/******/ 		};
/******/
/******/ 		// Execute the module function
/******/ 		modules[moduleId].call(module.exports, module, module.exports, __webpack_require__);
/******/
/******/ 		// Flag the module as loaded
/******/ 		module.l = true;
/******/
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/
/******/ 	// This file contains only the entry chunk.
/******/ 	// The chunk loading function for additional chunks
/******/ 	__webpack_require__.e = function requireEnsure(chunkId) {
/******/ 		var promises = [];
/******/
/******/
/******/ 		// JSONP chunk loading for javascript
/******/
/******/ 		var installedChunkData = installedChunks[chunkId];
/******/ 		if(installedChunkData !== 0) { // 0 means "already installed".
/******/
/******/ 			// a Promise means "currently loading".
/******/ 			if(installedChunkData) {
/******/ 				promises.push(installedChunkData[2]);
/******/ 			} else {
/******/ 				// setup Promise in chunk cache
/******/ 				var promise = new Promise(function(resolve, reject) {
/******/ 					installedChunkData = installedChunks[chunkId] = [resolve, reject];
/******/ 				});
/******/ 				promises.push(installedChunkData[2] = promise);
/******/
/******/ 				// start chunk loading
/******/ 				var script = document.createElement('script');
/******/ 				var onScriptComplete;
/******/
/******/ 				script.charset = 'utf-8';
/******/ 				script.timeout = 120;
/******/ 				if (__webpack_require__.nc) {
/******/ 					script.setAttribute("nonce", __webpack_require__.nc);
/******/ 				}
/******/ 				script.src = jsonpScriptSrc(chunkId);
/******/
/******/ 				// create error before stack unwound to get useful stacktrace later
/******/ 				var error = new Error();
/******/ 				onScriptComplete = function (event) {
/******/ 					// avoid mem leaks in IE.
/******/ 					script.onerror = script.onload = null;
/******/ 					clearTimeout(timeout);
/******/ 					var chunk = installedChunks[chunkId];
/******/ 					if(chunk !== 0) {
/******/ 						if(chunk) {
/******/ 							var errorType = event && (event.type === 'load' ? 'missing' : event.type);
/******/ 							var realSrc = event && event.target && event.target.src;
/******/ 							error.message = 'Loading chunk ' + chunkId + ' failed.\n(' + errorType + ': ' + realSrc + ')';
/******/ 							error.name = 'ChunkLoadError';
/******/ 							error.type = errorType;
/******/ 							error.request = realSrc;
/******/ 							chunk[1](error);
/******/ 						}
/******/ 						installedChunks[chunkId] = undefined;
/******/ 					}
/******/ 				};
/******/ 				var timeout = setTimeout(function(){
/******/ 					onScriptComplete({ type: 'timeout', target: script });
/******/ 				}, 120000);
/******/ 				script.onerror = script.onload = onScriptComplete;
/******/ 				document.head.appendChild(script);
/******/ 			}
/******/ 		}
/******/
/******/ 		// Fetch + compile chunk loading for webassembly
/******/
/******/ 		var wasmModules = {"1":["./pkg/index_bg.wasm"]}[chunkId] || [];
/******/
/******/ 		wasmModules.forEach(function(wasmModuleId) {
/******/ 			var installedWasmModuleData = installedWasmModules[wasmModuleId];
/******/
/******/ 			// a Promise means "currently loading" or "already loaded".
/******/ 			if(installedWasmModuleData)
/******/ 				promises.push(installedWasmModuleData);
/******/ 			else {
/******/ 				var importObject = wasmImportObjects[wasmModuleId]();
/******/ 				var req = fetch(__webpack_require__.p + "" + {"./pkg/index_bg.wasm":"595d207832b6eaa27fa3"}[wasmModuleId] + ".module.wasm");
/******/ 				var promise;
/******/ 				if(importObject instanceof Promise && typeof WebAssembly.compileStreaming === 'function') {
/******/ 					promise = Promise.all([WebAssembly.compileStreaming(req), importObject]).then(function(items) {
/******/ 						return WebAssembly.instantiate(items[0], items[1]);
/******/ 					});
/******/ 				} else if(typeof WebAssembly.instantiateStreaming === 'function') {
/******/ 					promise = WebAssembly.instantiateStreaming(req, importObject);
/******/ 				} else {
/******/ 					var bytesPromise = req.then(function(x) { return x.arrayBuffer(); });
/******/ 					promise = bytesPromise.then(function(bytes) {
/******/ 						return WebAssembly.instantiate(bytes, importObject);
/******/ 					});
/******/ 				}
/******/ 				promises.push(installedWasmModules[wasmModuleId] = promise.then(function(res) {
/******/ 					return __webpack_require__.w[wasmModuleId] = (res.instance || res).exports;
/******/ 				}));
/******/ 			}
/******/ 		});
/******/ 		return Promise.all(promises);
/******/ 	};
/******/
/******/ 	// expose the modules object (__webpack_modules__)
/******/ 	__webpack_require__.m = modules;
/******/
/******/ 	// expose the module cache
/******/ 	__webpack_require__.c = installedModules;
/******/
/******/ 	// define getter function for harmony exports
/******/ 	__webpack_require__.d = function(exports, name, getter) {
/******/ 		if(!__webpack_require__.o(exports, name)) {
/******/ 			Object.defineProperty(exports, name, { enumerable: true, get: getter });
/******/ 		}
/******/ 	};
/******/
/******/ 	// define __esModule on exports
/******/ 	__webpack_require__.r = function(exports) {
/******/ 		if(typeof Symbol !== 'undefined' && Symbol.toStringTag) {
/******/ 			Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
/******/ 		}
/******/ 		Object.defineProperty(exports, '__esModule', { value: true });
/******/ 	};
/******/
/******/ 	// create a fake namespace object
/******/ 	// mode & 1: value is a module id, require it
/******/ 	// mode & 2: merge all properties of value into the ns
/******/ 	// mode & 4: return value when already ns object
/******/ 	// mode & 8|1: behave like require
/******/ 	__webpack_require__.t = function(value, mode) {
/******/ 		if(mode & 1) value = __webpack_require__(value);
/******/ 		if(mode & 8) return value;
/******/ 		if((mode & 4) && typeof value === 'object' && value && value.__esModule) return value;
/******/ 		var ns = Object.create(null);
/******/ 		__webpack_require__.r(ns);
/******/ 		Object.defineProperty(ns, 'default', { enumerable: true, value: value });
/******/ 		if(mode & 2 && typeof value != 'string') for(var key in value) __webpack_require__.d(ns, key, function(key) { return value[key]; }.bind(null, key));
/******/ 		return ns;
/******/ 	};
/******/
/******/ 	// getDefaultExport function for compatibility with non-harmony modules
/******/ 	__webpack_require__.n = function(module) {
/******/ 		var getter = module && module.__esModule ?
/******/ 			function getDefault() { return module['default']; } :
/******/ 			function getModuleExports() { return module; };
/******/ 		__webpack_require__.d(getter, 'a', getter);
/******/ 		return getter;
/******/ 	};
/******/
/******/ 	// Object.prototype.hasOwnProperty.call
/******/ 	__webpack_require__.o = function(object, property) { return Object.prototype.hasOwnProperty.call(object, property); };
/******/
/******/ 	// __webpack_public_path__
/******/ 	__webpack_require__.p = "";
/******/
/******/ 	// on error function for async loading
/******/ 	__webpack_require__.oe = function(err) { console.error(err); throw err; };
/******/
/******/ 	// object with all WebAssembly.instance exports
/******/ 	__webpack_require__.w = {};
/******/
/******/ 	var jsonpArray = window["webpackJsonp"] = window["webpackJsonp"] || [];
/******/ 	var oldJsonpFunction = jsonpArray.push.bind(jsonpArray);
/******/ 	jsonpArray.push = webpackJsonpCallback;
/******/ 	jsonpArray = jsonpArray.slice();
/******/ 	for(var i = 0; i < jsonpArray.length; i++) webpackJsonpCallback(jsonpArray[i]);
/******/ 	var parentJsonpFunction = oldJsonpFunction;
/******/
/******/
/******/ 	// Load entry module and return exports
/******/ 	return __webpack_require__(__webpack_require__.s = "./bootstrap.js");
/******/ })
/************************************************************************/
/******/ ({

/***/ "./bootstrap.js":
/*!**********************!*\
  !*** ./bootstrap.js ***!
  \**********************/
/*! no exports provided */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony import */ var _styles_scss__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./styles.scss */ \"./styles.scss\");\n// https://github.com/webpack/webpack/issues/6615\nPromise.all(/*! import() */[__webpack_require__.e(0), __webpack_require__.e(1)]).then(__webpack_require__.t.bind(null, /*! ./index.ts */ \"./index.ts\", 7));\n\n\n\n\n//# sourceURL=webpack:///./bootstrap.js?");

/***/ }),

/***/ "./node_modules/css-loader/dist/cjs.js!./node_modules/sass-loader/dist/cjs.js!./styles.scss":
/*!**************************************************************************************************!*\
  !*** ./node_modules/css-loader/dist/cjs.js!./node_modules/sass-loader/dist/cjs.js!./styles.scss ***!
  \**************************************************************************************************/
/*! exports provided: default */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony import */ var _node_modules_css_loader_dist_runtime_api_js__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./node_modules/css-loader/dist/runtime/api.js */ \"./node_modules/css-loader/dist/runtime/api.js\");\n/* harmony import */ var _node_modules_css_loader_dist_runtime_api_js__WEBPACK_IMPORTED_MODULE_0___default = /*#__PURE__*/__webpack_require__.n(_node_modules_css_loader_dist_runtime_api_js__WEBPACK_IMPORTED_MODULE_0__);\n// Imports\n\nvar ___CSS_LOADER_EXPORT___ = _node_modules_css_loader_dist_runtime_api_js__WEBPACK_IMPORTED_MODULE_0___default()(function(i){return i[1]});\n// Module\n___CSS_LOADER_EXPORT___.push([module.i, \"kbd {\\n  border-radius: 3px;\\n  padding: 1px 2px 0;\\n  border: 1px solid black;\\n}\\n\\n#game-boy {\\n  display: grid;\\n  grid-template-columns: auto 100px 480px 100px auto;\\n  grid-template-rows: 50px 50px 216px 216px 50px 50px;\\n}\\n\\n#screen, #screen-overlay, .screen-background {\\n  grid-column: 3/4;\\n  grid-row: 3/5;\\n}\\n\\n#screen {\\n  display: block;\\n  margin: auto;\\n  width: 480px;\\n  height: 432px;\\n  outline-style: solid;\\n  z-index: 2;\\n}\\n\\n.screen-background {\\n  display: flex;\\n  background: white;\\n  z-index: 1;\\n}\\n\\n#screen-overlay {\\n  display: none;\\n  flex-direction: column;\\n  justify-content: center;\\n  align-items: center;\\n  text-align: center;\\n  z-index: 3;\\n  font-family: \\\"Press Start 2P\\\", monospace;\\n}\\n\\n.screen-border {\\n  display: inline-block;\\n  background-color: gray;\\n  border-radius: 25px 25px 100px 25px;\\n  grid-column: 2/5;\\n  grid-row: 2/6;\\n}\\n\\n.power {\\n  grid-column: 2/3;\\n  grid-row: 3/4;\\n  align-self: end;\\n  color: white;\\n  font-family: sans-serif;\\n  text-align: center;\\n  font-family: \\\"Merriweather Sans\\\", sans-serif;\\n}\\n\\n.power-light {\\n  margin: auto;\\n  margin-bottom: 10px;\\n  background: red;\\n  border-radius: 50%;\\n  height: 10px;\\n  width: 10px;\\n}\\n\\n.top-bar-container {\\n  display: flex;\\n  flex-direction: column;\\n  justify-content: center;\\n  text-align: right;\\n  font-family: sans-serif;\\n  grid-column: 3/4;\\n  grid-row: 2/3;\\n  font-family: \\\"Merriweather Sans\\\", sans-serif;\\n  color: white;\\n}\\n\\n.rom-container {\\n  text-align: center;\\n}\\n\\n.modal {\\n  display: none;\\n  position: fixed;\\n  z-index: 1;\\n  left: 0;\\n  top: 0;\\n  width: 100%;\\n  height: 100%;\\n  overflow: auto;\\n  background-color: rgba(0, 0, 0, 0.4);\\n}\\n\\n.modal-content {\\n  background-color: #fefefe;\\n  margin: 15% auto;\\n  padding: 20px;\\n  border: 1px solid #888;\\n  width: 80%;\\n}\\n\\n.close {\\n  color: #aaa;\\n  float: right;\\n  font-size: 28px;\\n  font-weight: bold;\\n}\", \"\"]);\n// Exports\n/* harmony default export */ __webpack_exports__[\"default\"] = (___CSS_LOADER_EXPORT___);\n\n\n//# sourceURL=webpack:///./styles.scss?./node_modules/css-loader/dist/cjs.js!./node_modules/sass-loader/dist/cjs.js");

/***/ }),

/***/ "./node_modules/css-loader/dist/runtime/api.js":
/*!*****************************************************!*\
  !*** ./node_modules/css-loader/dist/runtime/api.js ***!
  \*****************************************************/
/*! no static exports found */
/***/ (function(module, exports, __webpack_require__) {

"use strict";
eval("\n\n/*\n  MIT License http://www.opensource.org/licenses/mit-license.php\n  Author Tobias Koppers @sokra\n*/\n// css base code, injected by the css-loader\n// eslint-disable-next-line func-names\nmodule.exports = function (cssWithMappingToString) {\n  var list = []; // return the list of modules as css string\n\n  list.toString = function toString() {\n    return this.map(function (item) {\n      var content = cssWithMappingToString(item);\n\n      if (item[2]) {\n        return \"@media \".concat(item[2], \" {\").concat(content, \"}\");\n      }\n\n      return content;\n    }).join('');\n  }; // import a list of modules into the list\n  // eslint-disable-next-line func-names\n\n\n  list.i = function (modules, mediaQuery, dedupe) {\n    if (typeof modules === 'string') {\n      // eslint-disable-next-line no-param-reassign\n      modules = [[null, modules, '']];\n    }\n\n    var alreadyImportedModules = {};\n\n    if (dedupe) {\n      for (var i = 0; i < this.length; i++) {\n        // eslint-disable-next-line prefer-destructuring\n        var id = this[i][0];\n\n        if (id != null) {\n          alreadyImportedModules[id] = true;\n        }\n      }\n    }\n\n    for (var _i = 0; _i < modules.length; _i++) {\n      var item = [].concat(modules[_i]);\n\n      if (dedupe && alreadyImportedModules[item[0]]) {\n        // eslint-disable-next-line no-continue\n        continue;\n      }\n\n      if (mediaQuery) {\n        if (!item[2]) {\n          item[2] = mediaQuery;\n        } else {\n          item[2] = \"\".concat(mediaQuery, \" and \").concat(item[2]);\n        }\n      }\n\n      list.push(item);\n    }\n  };\n\n  return list;\n};\n\n//# sourceURL=webpack:///./node_modules/css-loader/dist/runtime/api.js?");

/***/ }),

/***/ "./node_modules/style-loader/dist/runtime/injectStylesIntoStyleTag.js":
/*!****************************************************************************!*\
  !*** ./node_modules/style-loader/dist/runtime/injectStylesIntoStyleTag.js ***!
  \****************************************************************************/
/*! no static exports found */
/***/ (function(module, exports, __webpack_require__) {

"use strict";
eval("\n\nvar isOldIE = function isOldIE() {\n  var memo;\n  return function memorize() {\n    if (typeof memo === 'undefined') {\n      // Test for IE <= 9 as proposed by Browserhacks\n      // @see http://browserhacks.com/#hack-e71d8692f65334173fee715c222cb805\n      // Tests for existence of standard globals is to allow style-loader\n      // to operate correctly into non-standard environments\n      // @see https://github.com/webpack-contrib/style-loader/issues/177\n      memo = Boolean(window && document && document.all && !window.atob);\n    }\n\n    return memo;\n  };\n}();\n\nvar getTarget = function getTarget() {\n  var memo = {};\n  return function memorize(target) {\n    if (typeof memo[target] === 'undefined') {\n      var styleTarget = document.querySelector(target); // Special case to return head of iframe instead of iframe itself\n\n      if (window.HTMLIFrameElement && styleTarget instanceof window.HTMLIFrameElement) {\n        try {\n          // This will throw an exception if access to iframe is blocked\n          // due to cross-origin restrictions\n          styleTarget = styleTarget.contentDocument.head;\n        } catch (e) {\n          // istanbul ignore next\n          styleTarget = null;\n        }\n      }\n\n      memo[target] = styleTarget;\n    }\n\n    return memo[target];\n  };\n}();\n\nvar stylesInDom = [];\n\nfunction getIndexByIdentifier(identifier) {\n  var result = -1;\n\n  for (var i = 0; i < stylesInDom.length; i++) {\n    if (stylesInDom[i].identifier === identifier) {\n      result = i;\n      break;\n    }\n  }\n\n  return result;\n}\n\nfunction modulesToDom(list, options) {\n  var idCountMap = {};\n  var identifiers = [];\n\n  for (var i = 0; i < list.length; i++) {\n    var item = list[i];\n    var id = options.base ? item[0] + options.base : item[0];\n    var count = idCountMap[id] || 0;\n    var identifier = \"\".concat(id, \" \").concat(count);\n    idCountMap[id] = count + 1;\n    var index = getIndexByIdentifier(identifier);\n    var obj = {\n      css: item[1],\n      media: item[2],\n      sourceMap: item[3]\n    };\n\n    if (index !== -1) {\n      stylesInDom[index].references++;\n      stylesInDom[index].updater(obj);\n    } else {\n      stylesInDom.push({\n        identifier: identifier,\n        updater: addStyle(obj, options),\n        references: 1\n      });\n    }\n\n    identifiers.push(identifier);\n  }\n\n  return identifiers;\n}\n\nfunction insertStyleElement(options) {\n  var style = document.createElement('style');\n  var attributes = options.attributes || {};\n\n  if (typeof attributes.nonce === 'undefined') {\n    var nonce =  true ? __webpack_require__.nc : undefined;\n\n    if (nonce) {\n      attributes.nonce = nonce;\n    }\n  }\n\n  Object.keys(attributes).forEach(function (key) {\n    style.setAttribute(key, attributes[key]);\n  });\n\n  if (typeof options.insert === 'function') {\n    options.insert(style);\n  } else {\n    var target = getTarget(options.insert || 'head');\n\n    if (!target) {\n      throw new Error(\"Couldn't find a style target. This probably means that the value for the 'insert' parameter is invalid.\");\n    }\n\n    target.appendChild(style);\n  }\n\n  return style;\n}\n\nfunction removeStyleElement(style) {\n  // istanbul ignore if\n  if (style.parentNode === null) {\n    return false;\n  }\n\n  style.parentNode.removeChild(style);\n}\n/* istanbul ignore next  */\n\n\nvar replaceText = function replaceText() {\n  var textStore = [];\n  return function replace(index, replacement) {\n    textStore[index] = replacement;\n    return textStore.filter(Boolean).join('\\n');\n  };\n}();\n\nfunction applyToSingletonTag(style, index, remove, obj) {\n  var css = remove ? '' : obj.media ? \"@media \".concat(obj.media, \" {\").concat(obj.css, \"}\") : obj.css; // For old IE\n\n  /* istanbul ignore if  */\n\n  if (style.styleSheet) {\n    style.styleSheet.cssText = replaceText(index, css);\n  } else {\n    var cssNode = document.createTextNode(css);\n    var childNodes = style.childNodes;\n\n    if (childNodes[index]) {\n      style.removeChild(childNodes[index]);\n    }\n\n    if (childNodes.length) {\n      style.insertBefore(cssNode, childNodes[index]);\n    } else {\n      style.appendChild(cssNode);\n    }\n  }\n}\n\nfunction applyToTag(style, options, obj) {\n  var css = obj.css;\n  var media = obj.media;\n  var sourceMap = obj.sourceMap;\n\n  if (media) {\n    style.setAttribute('media', media);\n  } else {\n    style.removeAttribute('media');\n  }\n\n  if (sourceMap && typeof btoa !== 'undefined') {\n    css += \"\\n/*# sourceMappingURL=data:application/json;base64,\".concat(btoa(unescape(encodeURIComponent(JSON.stringify(sourceMap)))), \" */\");\n  } // For old IE\n\n  /* istanbul ignore if  */\n\n\n  if (style.styleSheet) {\n    style.styleSheet.cssText = css;\n  } else {\n    while (style.firstChild) {\n      style.removeChild(style.firstChild);\n    }\n\n    style.appendChild(document.createTextNode(css));\n  }\n}\n\nvar singleton = null;\nvar singletonCounter = 0;\n\nfunction addStyle(obj, options) {\n  var style;\n  var update;\n  var remove;\n\n  if (options.singleton) {\n    var styleIndex = singletonCounter++;\n    style = singleton || (singleton = insertStyleElement(options));\n    update = applyToSingletonTag.bind(null, style, styleIndex, false);\n    remove = applyToSingletonTag.bind(null, style, styleIndex, true);\n  } else {\n    style = insertStyleElement(options);\n    update = applyToTag.bind(null, style, options);\n\n    remove = function remove() {\n      removeStyleElement(style);\n    };\n  }\n\n  update(obj);\n  return function updateStyle(newObj) {\n    if (newObj) {\n      if (newObj.css === obj.css && newObj.media === obj.media && newObj.sourceMap === obj.sourceMap) {\n        return;\n      }\n\n      update(obj = newObj);\n    } else {\n      remove();\n    }\n  };\n}\n\nmodule.exports = function (list, options) {\n  options = options || {}; // Force single-tag solution on IE6-9, which has a hard limit on the # of <style>\n  // tags it will allow on a page\n\n  if (!options.singleton && typeof options.singleton !== 'boolean') {\n    options.singleton = isOldIE();\n  }\n\n  list = list || [];\n  var lastIdentifiers = modulesToDom(list, options);\n  return function update(newList) {\n    newList = newList || [];\n\n    if (Object.prototype.toString.call(newList) !== '[object Array]') {\n      return;\n    }\n\n    for (var i = 0; i < lastIdentifiers.length; i++) {\n      var identifier = lastIdentifiers[i];\n      var index = getIndexByIdentifier(identifier);\n      stylesInDom[index].references--;\n    }\n\n    var newLastIdentifiers = modulesToDom(newList, options);\n\n    for (var _i = 0; _i < lastIdentifiers.length; _i++) {\n      var _identifier = lastIdentifiers[_i];\n\n      var _index = getIndexByIdentifier(_identifier);\n\n      if (stylesInDom[_index].references === 0) {\n        stylesInDom[_index].updater();\n\n        stylesInDom.splice(_index, 1);\n      }\n    }\n\n    lastIdentifiers = newLastIdentifiers;\n  };\n};\n\n//# sourceURL=webpack:///./node_modules/style-loader/dist/runtime/injectStylesIntoStyleTag.js?");

/***/ }),

/***/ "./styles.scss":
/*!*********************!*\
  !*** ./styles.scss ***!
  \*********************/
/*! exports provided: default */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony import */ var _node_modules_style_loader_dist_runtime_injectStylesIntoStyleTag_js__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./node_modules/style-loader/dist/runtime/injectStylesIntoStyleTag.js */ \"./node_modules/style-loader/dist/runtime/injectStylesIntoStyleTag.js\");\n/* harmony import */ var _node_modules_style_loader_dist_runtime_injectStylesIntoStyleTag_js__WEBPACK_IMPORTED_MODULE_0___default = /*#__PURE__*/__webpack_require__.n(_node_modules_style_loader_dist_runtime_injectStylesIntoStyleTag_js__WEBPACK_IMPORTED_MODULE_0__);\n/* harmony import */ var _node_modules_css_loader_dist_cjs_js_node_modules_sass_loader_dist_cjs_js_styles_scss__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! !./node_modules/css-loader/dist/cjs.js!./node_modules/sass-loader/dist/cjs.js!./styles.scss */ \"./node_modules/css-loader/dist/cjs.js!./node_modules/sass-loader/dist/cjs.js!./styles.scss\");\n\n            \n\nvar options = {};\n\noptions.insert = \"head\";\noptions.singleton = false;\n\nvar update = _node_modules_style_loader_dist_runtime_injectStylesIntoStyleTag_js__WEBPACK_IMPORTED_MODULE_0___default()(_node_modules_css_loader_dist_cjs_js_node_modules_sass_loader_dist_cjs_js_styles_scss__WEBPACK_IMPORTED_MODULE_1__[\"default\"], options);\n\n\n\n/* harmony default export */ __webpack_exports__[\"default\"] = (_node_modules_css_loader_dist_cjs_js_node_modules_sass_loader_dist_cjs_js_styles_scss__WEBPACK_IMPORTED_MODULE_1__[\"default\"].locals || {});\n\n//# sourceURL=webpack:///./styles.scss?");

/***/ })

/******/ });