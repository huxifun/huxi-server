$(document).ready(function(){
    $.get('/user/status', function(res){
        if (res != "") {
            $("#login").html(
                '<span class="">' + res + '</span> <a class="" href="/user/logout">退出</a>'
            );
        } else {
          $("#login").html(
            '<a href="/user/reg" class="btn btn-primary mx-1">注册</a><a href="/user/login" class="btn btn-primary">登录</a>'
          )
        }
    });
});
