package com.workorder.launcher;

import com.workorder.WorkOrderApplication;
import org.springframework.boot.SpringApplication;
import org.springframework.context.ConfigurableApplicationContext;

import java.awt.Desktop;
import java.io.IOException;
import java.net.HttpURLConnection;
import java.net.URI;
import java.net.URL;
import java.util.concurrent.TimeUnit;

/**
 * 桌面启动器：拉起 Spring Boot 并以 app 模式打开浏览器。
 */
public class DesktopLauncher {

    public static void main(String[] args) throws Exception {
        SpringApplication application = new SpringApplication(WorkOrderApplication.class);
        application.addInitializers(new com.workorder.config.DataDirectoryInitializer());
        ConfigurableApplicationContext context = application.run(args);
        int port = context.getEnvironment().getProperty("server.port", Integer.class, 8080);
        waitForPort(port, 60);
        openBrowserAppMode("http://localhost:" + port);
    }

    static void waitForPort(int port, int timeoutSeconds) throws InterruptedException {
        long deadline = System.currentTimeMillis() + TimeUnit.SECONDS.toMillis(timeoutSeconds);
        while (System.currentTimeMillis() < deadline) {
            if (isPortReady(port)) {
                return;
            }
            Thread.sleep(500);
        }
        throw new IllegalStateException("Application did not start on port " + port);
    }

    static boolean isPortReady(int port) {
        try {
            URL url = new URL("http://localhost:" + port);
            HttpURLConnection connection = (HttpURLConnection) url.openConnection();
            connection.setConnectTimeout(1000);
            connection.setReadTimeout(1000);
            connection.setRequestMethod("GET");
            int code = connection.getResponseCode();
            connection.disconnect();
            return code > 0;
        } catch (IOException ex) {
            return false;
        }
    }

    static void openBrowserAppMode(String url) throws IOException {
        if (tryChromeAppMode(url) || tryEdgeAppMode(url)) {
            return;
        }
        if (Desktop.isDesktopSupported()) {
            Desktop.getDesktop().browse(URI.create(url));
        }
    }

    private static boolean tryChromeAppMode(String url) {
        return runBrowserCommand(new String[]{
                "cmd", "/c", "start", "", "chrome", "--app=" + url
        });
    }

    private static boolean tryEdgeAppMode(String url) {
        return runBrowserCommand(new String[]{
                "cmd", "/c", "start", "", "msedge", "--app=" + url
        });
    }

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
