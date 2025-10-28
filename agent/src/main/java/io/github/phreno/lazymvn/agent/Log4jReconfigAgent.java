package io.github.phreno.lazymvn.agent;

import java.lang.instrument.Instrumentation;
import java.lang.reflect.Method;
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
                // Monitor Log4j configuration and reconfigure only when needed
                // This prevents appender closing issues from multiple reconfigurations
                for (int attempt = 1; attempt <= 5; attempt++) {
                    Thread.sleep(2000); // Wait 2 seconds between each check
                    
                    // Check if reconfiguration is needed by comparing current config
                    if (isReconfigurationNeeded(configUrl)) {
                        System.err.println("[LazyMVN Agent] Reconfiguration needed (attempt " + attempt + "/5)...");
                        reconfigureLog4j(configUrl);
                    } else {
                        System.err.println("[LazyMVN Agent] Configuration check " + attempt + "/5 - LazyMVN config already active, skipping reconfiguration");
                    }
                    
                    if (attempt == 5) {
                        System.err.println("[LazyMVN Agent] Final check completed");
                        System.err.println("[LazyMVN Agent] LazyMVN log configuration is active");
                    }
                }
            } catch (InterruptedException e) {
                System.err.println("[LazyMVN Agent] Thread interrupted: " + e.getMessage());
            }
        }

        /**
         * Check if Log4j reconfiguration is needed by comparing current configuration
         * with the expected LazyMVN configuration.
         * 
         * @param configUrl URL to the LazyMVN Log4j configuration file
         * @return true if reconfiguration is needed, false if LazyMVN config is already active
         */
        private boolean isReconfigurationNeeded(String configUrl) {
            try {
                // Get current Log4j configuration via reflection
                Class<?> logManagerClass = Class.forName("org.apache.log4j.LogManager");
                
                // Get root logger to inspect current configuration
                Method getRootLoggerMethod = logManagerClass.getMethod("getRootLogger");
                Object rootLogger = getRootLoggerMethod.invoke(null);
                
                // Get current appenders
                Method getAllAppendersMethod = rootLogger.getClass().getMethod("getAllAppenders");
                Object appenders = getAllAppendersMethod.invoke(rootLogger);
                
                // Check if appenders exist (if not, reconfiguration needed)
                Method hasMoreElementsMethod = appenders.getClass().getMethod("hasMoreElements");
                boolean hasAppenders = (Boolean) hasMoreElementsMethod.invoke(appenders);
                
                if (!hasAppenders) {
                    System.err.println("[LazyMVN Agent] No appenders found - reconfiguration needed");
                    return true;
                }
                
                // Get the first appender to check its layout
                Method nextElementMethod = appenders.getClass().getMethod("nextElement");
                Object appender = nextElementMethod.invoke(appenders);
                
                // Get the layout from the appender
                Method getLayoutMethod = appender.getClass().getMethod("getLayout");
                Object layout = getLayoutMethod.invoke(appender);
                
                if (layout == null) {
                    System.err.println("[LazyMVN Agent] No layout found - reconfiguration needed");
                    return true;
                }
                
                // Get the conversion pattern from the layout
                Method getConversionPatternMethod = layout.getClass().getMethod("getConversionPattern");
                String currentPattern = (String) getConversionPatternMethod.invoke(layout);
                
                // Expected pattern from lazymvn.toml: "[%p][%c] %m%n"
                String expectedPattern = "[%p][%c] %m%n";
                
                if (currentPattern != null && currentPattern.equals(expectedPattern)) {
                    System.err.println("[LazyMVN Agent] LazyMVN pattern detected: " + currentPattern);
                    return false; // Configuration already correct
                } else {
                    System.err.println("[LazyMVN Agent] Different pattern detected: " + currentPattern + " (expected: " + expectedPattern + ")");
                    return true; // Reconfiguration needed
                }
                
            } catch (Exception e) {
                System.err.println("[LazyMVN Agent] Could not inspect current Log4j configuration: " + e.getMessage());
                // If we can't inspect, better to reconfigure
                return true;
            }
        }

        /**
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
