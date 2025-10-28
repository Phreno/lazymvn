package io.github.phreno.lazymvn.agent;

import java.lang.instrument.Instrumentation;
import java.net.URL;

/**
 * Java Agent that forces Log4j 1.x reconfiguration after application initialization.
 * 
 * This agent solves the problem where custom factories (like Log4jJbossLoggerFactory)
 * reinitialize Log4j with their own configuration, overwriting the configuration
 * provided via -Dlog4j.configuration system property.
 * 
 * Strategy:
 * 1. JVM starts with JAVA_TOOL_OPTIONS → Log4j loads LazyMVN config ✓
 * 2. Application starts → Custom factory reloads its config ✗ (overwrites LazyMVN)
 * 3. This agent waits 2 seconds → Forces reconfiguration with LazyMVN config ✓
 * 
 * Usage:
 *   java -javaagent:log4j-reconfig-agent.jar -Dlog4j.configuration=file:///path/to/config.properties ...
 */
public class Log4jReconfigAgent {

    /**
     * Agent entry point (called before main method).
     * 
     * @param agentArgs Arguments passed to the agent (unused)
     * @param inst Instrumentation instance (unused)
     */
    public static void premain(String agentArgs, Instrumentation inst) {
        System.err.println("[LazyMVN Agent] Starting Log4j reconfiguration agent...");
        
        // Get Log4j configuration URL from system properties
        String log4jConfig = System.getProperty("log4j.configuration");
        
        if (log4jConfig == null || log4jConfig.isEmpty()) {
            System.err.println("[LazyMVN Agent] No log4j.configuration system property found, agent disabled.");
            return;
        }
        
        System.err.println("[LazyMVN Agent] Will reconfigure Log4j with: " + log4jConfig);
        
        // Start background thread to reconfigure Log4j after application initialization
        Thread reconfigThread = new Thread(new Log4jReconfigurator(log4jConfig), "lazymvn-log4j-reconfig");
        reconfigThread.setDaemon(true); // Don't prevent JVM shutdown
        reconfigThread.start();
    }
    
    /**
     * Runnable that waits for application initialization and then reconfigures Log4j.
     */
    private static class Log4jReconfigurator implements Runnable {
        private final String configUrl;
        
        public Log4jReconfigurator(String configUrl) {
            this.configUrl = configUrl;
        }
        
        @Override
        public void run() {
            try {
                // Force reconfiguration multiple times to override any late Log4j reconfigurations
                // This ensures LazyMVN config persists even if Log4jJbossLoggerFactory reconfigures late
                for (int attempt = 1; attempt <= 5; attempt++) {
                    Thread.sleep(2000); // Wait 2 seconds between each attempt
                    
                    System.err.println("[LazyMVN Agent] Reconfiguration attempt " + attempt + "/5...");
                    reconfigureLog4j(configUrl);
                    
                    if (attempt == 5) {
                        System.err.println("[LazyMVN Agent] Final reconfiguration completed");
                        System.err.println("[LazyMVN Agent] LazyMVN log configuration should now persist");
                    }
                }
            } catch (InterruptedException e) {
                System.err.println("[LazyMVN Agent] Thread interrupted: " + e.getMessage());
            }
        }        /**
         * Reconfigures Log4j with the specified configuration URL.
         * Uses reflection to avoid compile-time dependency on Log4j.
         */
        private void reconfigureLog4j(String configUrl) {
            try {
                // Convert file:/// URL to actual URL object
                URL url = new URL(configUrl);
                
                // Use reflection to call PropertyConfigurator.configure(URL)
                // This avoids compile-time dependency on Log4j 1.x
                Class<?> configuratorClass = Class.forName("org.apache.log4j.PropertyConfigurator");
                java.lang.reflect.Method configureMethod = configuratorClass.getMethod("configure", URL.class);
                configureMethod.invoke(null, url);
                
                System.err.println("[LazyMVN Agent] ✓ Log4j successfully reconfigured with LazyMVN config");
                System.err.println("[LazyMVN Agent] ✓ Log format and levels from lazymvn.toml are now active");
                
            } catch (ClassNotFoundException e) {
                System.err.println("[LazyMVN Agent] Log4j not found in classpath (this is normal if not using Log4j 1.x)");
            } catch (NoSuchMethodException e) {
                System.err.println("[LazyMVN Agent] PropertyConfigurator.configure(URL) method not found");
            } catch (Exception e) {
                System.err.println("[LazyMVN Agent] Error during reconfiguration: " + e.getMessage());
                e.printStackTrace();
            }
        }
    }
}
