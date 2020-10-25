$(function() {
  // click event: wrap in form input
  $(document).on("click", "td.str", function() {
    if ($(this).find("input").length > 0) {
      return;
    }

    var inner_val = $(this).text();
    var wrapped_val = $("<input type=\"text\"/>").val(inner_val).data("orig-val", inner_val);
    $(this).html(wrapped_val);
  });

  // input unfocus
  $(document).on("blur", "td.str input", function() {
    var inner_val = $(this).val();
    var orig_val = $(this).data("orig-val");
    if (inner_val != orig_val) {
      // TODO AJAX
      document.location.reload(true);
    }
    $(this).replaceWith(inner_val);
    return true;
  });
});
