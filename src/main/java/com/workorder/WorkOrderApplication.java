package com.workorder;

import com.workorder.config.DataDirectoryInitializer;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;

/**
 * workOrder 应用入口。
 */
@SpringBootApplication
public class WorkOrderApplication {

    public static void main(String[] args) {
        SpringApplication application = new SpringApplication(WorkOrderApplication.class);
        application.addInitializers(new DataDirectoryInitializer());
        application.run(args);
    }
}
