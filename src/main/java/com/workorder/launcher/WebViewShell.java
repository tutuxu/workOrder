package com.workorder.launcher;

import javafx.application.Application;
import javafx.application.Platform;
import javafx.scene.Scene;
import javafx.scene.web.WebView;
import javafx.stage.Stage;
import org.springframework.context.ConfigurableApplicationContext;

/**
 * JavaFX WebView 桌面窗口：内嵌加载本地 Vaadin 页面。
 */
public class WebViewShell extends Application {

    private static final int DEFAULT_WIDTH = 1200;
    private static final int DEFAULT_HEIGHT = 800;
    private static final int MIN_WIDTH = 800;
    private static final int MIN_HEIGHT = 600;

    private static String appUrl;
    private static ConfigurableApplicationContext springContext;

    /**
     * 配置 WebView 要加载的地址与 Spring 上下文（关闭窗口时用于优雅停机）。
     *
     * @param url            本地服务地址，如 http://localhost:8081
     * @param springContext  Spring Boot 应用上下文
     */
    static void configure(String url, ConfigurableApplicationContext springContext) {
        appUrl = url;
        WebViewShell.springContext = springContext;
    }

    @Override
    public void start(Stage stage) {
        // 1) 创建 WebView 并加载应用地址
        WebView webView = new WebView();
        webView.getEngine().load(appUrl);

        // 2) 组装场景与窗口属性
        Scene scene = new Scene(webView, DEFAULT_WIDTH, DEFAULT_HEIGHT);
        stage.setTitle("workOrder");
        stage.setMinWidth(MIN_WIDTH);
        stage.setMinHeight(MIN_HEIGHT);
        stage.setScene(scene);
        stage.setOnCloseRequest(event -> shutdown());
        stage.show();
    }

    /**
     * 关闭窗口时停止 Spring Boot 并退出 JavaFX 运行时。
     */
    private void shutdown() {
        // 1) 关闭 Spring 上下文
        if (springContext != null && springContext.isActive()) {
            springContext.close();
        }
        // 2) 退出 JavaFX
        Platform.exit();
    }
}
