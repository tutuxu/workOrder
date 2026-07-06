package com.workorder.launcher;

import com.workorder.WorkOrderApplication;
import com.workorder.config.DataDirectoryInitializer;
import javafx.application.Application;
import org.springframework.boot.SpringApplication;
import org.springframework.context.ConfigurableApplicationContext;

import java.awt.Desktop;
import java.io.IOException;
import java.net.HttpURLConnection;
import java.net.URI;
import java.net.URL;
import java.util.concurrent.TimeUnit;

/**
 * 桌面启动器：拉起 Spring Boot 并以 JavaFX WebView 打开工作窗口。
 */
public class DesktopLauncher {

    private static final int DEFAULT_PORT = 8081;
    private static final int STARTUP_TIMEOUT_SECONDS = 60;

    public static void main(String[] args) throws Exception {
        // 1) 后台启动 Spring Boot
        ConfigurableApplicationContext context = startSpringBoot(args);
        // 服务监听端口（与 application.properties 中 server.port 一致）
        int port = context.getEnvironment().getProperty("server.port", Integer.class, DEFAULT_PORT);
        // 2) 等待 HTTP 服务就绪
        waitForPort(port, STARTUP_TIMEOUT_SECONDS);
        // 本地应用访问地址
        String appUrl = "http://localhost:" + port;
        // 3) 打开桌面窗口
        openDesktopWindow(appUrl, context);
    }

    /**
     * 在后台线程启动 Spring Boot 并返回应用上下文。
     *
     * @param args Spring Boot 启动参数
     * @return 已启动的 Spring 应用上下文
     */
    private static ConfigurableApplicationContext startSpringBoot(String[] args) {
        SpringApplication application = new SpringApplication(WorkOrderApplication.class);
        application.addInitializers(new DataDirectoryInitializer());
        return application.run(args);
    }

    /**
     * 优先使用 JavaFX WebView 打开窗口；失败时回退到浏览器 app 模式。
     *
     * @param appUrl  本地服务地址
     * @param context Spring Boot 应用上下文
     */
    private static void openDesktopWindow(String appUrl, ConfigurableApplicationContext context)
            throws IOException {
        try {
            WebViewShell.configure(appUrl, context);
            Application.launch(WebViewShell.class);
        } catch (IllegalStateException ex) {
            openBrowserAppMode(appUrl);
        }
    }

    /**
     * 轮询直到指定端口的 HTTP 服务可访问。
     *
     * @param port           监听端口
     * @param timeoutSeconds 超时秒数
     * @throws InterruptedException 等待被中断
     */
    static void waitForPort(int port, int timeoutSeconds) throws InterruptedException {
        // 轮询截止时间
        long deadline = System.currentTimeMillis() + TimeUnit.SECONDS.toMillis(timeoutSeconds);
        // 在超时前反复探测端口
        while (System.currentTimeMillis() < deadline) {
            if (isPortReady(port)) {
                return;
            }
            Thread.sleep(500);
        }
        throw new IllegalStateException("Application did not start on port " + port);
    }

    /**
     * 探测本地 HTTP 端口是否已响应。
     *
     * @param port 监听端口
     * @return 端口可访问时返回 true
     */
    static boolean isPortReady(int port) {
        try {
            URL url = new URL("http://localhost:" + port);
            HttpURLConnection connection = (HttpURLConnection) url.openConnection();
            connection.setConnectTimeout(1000);
            connection.setReadTimeout(1000);
            connection.setRequestMethod("GET");
            // HTTP 状态码（任意有效响应即视为就绪）
            int code = connection.getResponseCode();
            connection.disconnect();
            return code > 0;
        } catch (IOException ex) {
            return false;
        }
    }

    /**
     * 以 Chrome/Edge app 模式或系统默认浏览器打开地址（WebView 不可用时的回退）。
     *
     * @param url 要打开的地址
     * @throws IOException 系统浏览器调用失败
     */
    static void openBrowserAppMode(String url) throws IOException {
        if (tryChromeAppMode(url) || tryEdgeAppMode(url)) {
            return;
        }
        if (Desktop.isDesktopSupported()) {
            Desktop.getDesktop().browse(URI.create(url));
        }
    }

    /**
     * 尝试用 Chrome app 模式打开地址。
     *
     * @param url 要打开的地址
     * @return 启动成功返回 true
     */
    private static boolean tryChromeAppMode(String url) {
        return runBrowserCommand(new String[]{
                "cmd", "/c", "start", "", "chrome", "--app=" + url
        });
    }

    /**
     * 尝试用 Edge app 模式打开地址。
     *
     * @param url 要打开的地址
     * @return 启动成功返回 true
     */
    private static boolean tryEdgeAppMode(String url) {
        return runBrowserCommand(new String[]{
                "cmd", "/c", "start", "", "msedge", "--app=" + url
        });
    }

    /**
     * 执行外部浏览器启动命令。
     *
     * @param command 进程命令行
     * @return 进程启动成功返回 true
     */
    private static boolean runBrowserCommand(String[] command) {
        try {
            Process process = new ProcessBuilder(command).start();
            process.getInputStream().close();
            process.getErrorStream().close();
            return process.waitFor(3, TimeUnit.SECONDS) || process.isAlive();
        } catch (Exception ex) {
            return false;
        }
    }
}
