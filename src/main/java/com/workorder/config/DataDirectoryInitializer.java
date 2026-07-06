package com.workorder.config;

import org.springframework.context.ApplicationContextInitializer;
import org.springframework.context.ConfigurableApplicationContext;
import org.springframework.core.env.Environment;
import org.springframework.core.env.MapPropertySource;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.HashMap;
import java.util.Map;

/**
 * 在 Flyway / DataSource 初始化前解析并创建项目内 data 目录。
 */
public class DataDirectoryInitializer implements ApplicationContextInitializer<ConfigurableApplicationContext> {

    static final String DATA_DIR_PROPERTY = "workorder.data.dir";

    @Override
    public void initialize(ConfigurableApplicationContext applicationContext) {
        Environment environment = applicationContext.getEnvironment();
        Path dataDir = resolveDataDirectory(environment);
        Path dbFile = dataDir.resolve("workorder.db");

        try {
            Files.createDirectories(dataDir);
        } catch (IOException ex) {
            throw new IllegalStateException("Failed to create data directory: " + dataDir, ex);
        }

        Map<String, Object> properties = new HashMap<>();
        properties.put("spring.datasource.url", "jdbc:sqlite:" + toJdbcPath(dbFile));
        applicationContext.getEnvironment().getPropertySources()
                .addFirst(new MapPropertySource("workorderDataDirectory", properties));
    }

    static Path resolveDataDirectory(Environment environment) {
        String configuredDir = environment.getProperty(DATA_DIR_PROPERTY);
        if (configuredDir != null && !configuredDir.isBlank()) {
            Path configured = Paths.get(configuredDir.trim());
            return configured.isAbsolute()
                    ? configured.normalize()
                    : Paths.get(System.getProperty("user.dir")).resolve(configured).normalize();
        }

        String datasourceUrl = environment.getProperty("spring.datasource.url");
        if (datasourceUrl != null && datasourceUrl.startsWith("jdbc:sqlite:")) {
            Path dbPath = Paths.get(datasourceUrl.substring("jdbc:sqlite:".length()));
            Path parent = dbPath.getParent();
            if (parent != null) {
                if (parent.isAbsolute()) {
                    return parent.normalize();
                }
                return Paths.get(System.getProperty("user.dir")).resolve(parent).normalize();
            }
        }

        return Paths.get(System.getProperty("user.dir")).resolve("data").normalize();
    }

    private static String toJdbcPath(Path path) {
        return path.toAbsolutePath().normalize().toString().replace('\\', '/');
    }
}
