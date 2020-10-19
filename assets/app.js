$(function() {
  // click event: wrap in form input
  $("td.str").click(function() {
    if ($(this).find("input").length > 0) {
      return true;
    }

    var inner_val = $(this).text();
    var wrapped_val = $("<input type=\"text\"/>").val(inner_val);
    $(this).html(wrapped_val);
    return false;
  });

  // input unfocus
  $("body").on("blur", "td.str input", function() {
    var inner_val = $(this).val();
    $(this).replaceWith(inner_val);
    return true;
  });
});
